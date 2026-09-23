use crate::core::ast::{
    Action, ComparisonOperator, Condition, PointsFormula, Rule, RuleProgram,
};
use crate::core::audit::{AuditReport, CategoryScoreBreakdown, FiredRuleRecord};
use crate::core::context::FactContext;
use crate::core::error::{EngineError, Result};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// The primary rule evaluation engine
pub struct Engine {
    program: RuleProgram,
}

impl Engine {
    /// Create a new Engine with a given RuleProgram
    pub fn new(program: RuleProgram) -> Self {
        Self { program }
    }

    /// Access the underlying RuleProgram
    pub fn program(&self) -> &RuleProgram {
        &self.program
    }

    /// Evaluate an applicant / fact context against the rules program
    pub fn evaluate(&self, initial_context: &FactContext) -> Result<AuditReport> {
        let mut ctx = initial_context.clone();
        let mut category_raw_scores: HashMap<String, f64> = HashMap::new();
        let mut fired_rules: Vec<FiredRuleRecord> = Vec::new();
        let mut ineligibility_reasons: Vec<String> = Vec::new();
        let mut overall_eligible = true;
        let mut fired_activation_groups: HashSet<String> = HashSet::new();

        // Applicant ID extraction for report header if available
        let applicant_id = ctx.get_str("applicant.id").ok().map(|s| s.to_string());

        // Standard phase ordering
        let phase_order = ["validation", "enrichment", "scoring", "capping", "verdict"];
        
        // Group enabled rules by phase
        let mut rules_by_phase: HashMap<String, Vec<&Rule>> = HashMap::new();
        for rule in &self.program.rules {
            if rule.enabled {
                rules_by_phase
                    .entry(rule.phase.clone())
                    .or_default()
                    .push(rule);
            }
        }

        // Execute phase by phase
        for phase in phase_order {
            if let Some(rules) = rules_by_phase.get_mut(phase) {
                // Sort by priority descending (salience), tie-break deterministically by ID
                rules.sort_by(|a, b| {
                    b.priority
                        .cmp(&a.priority)
                        .then_with(|| a.id.cmp(&b.id))
                });

                for rule in rules {
                    // Check activation group (XOR mutual exclusion)
                    if let Some(ref group) = rule.activation_group {
                        if fired_activation_groups.contains(group) {
                            // Another rule in this activation group already fired; skip
                            continue;
                        }
                    }

                    // Evaluate condition
                    let passed = self.evaluate_condition(&ctx, &rule.condition)?;
                    if passed {
                        // Mark activation group as fired
                        if let Some(ref group) = rule.activation_group {
                            fired_activation_groups.insert(group.clone());
                        }

                        let mut rule_awarded_points_or_eligibility = false;

                        // Execute actions
                        for action in &rule.actions {
                            if matches!(action, Action::AwardPoints { .. } | Action::SetEligibility { .. }) {
                                rule_awarded_points_or_eligibility = true;
                            }
                            self.execute_action(
                                action,
                                rule,
                                &mut ctx,
                                &mut category_raw_scores,
                                &mut fired_rules,
                                &mut overall_eligible,
                                &mut ineligibility_reasons,
                            )?;
                        }

                        // If rule had no AwardPoints or SetEligibility action, still log it in fired_rules
                        if !rule_awarded_points_or_eligibility {
                            fired_rules.push(FiredRuleRecord {
                                rule_id: rule.id.clone(),
                                rule_name: rule.name.clone(),
                                category: rule.category.clone(),
                                points_awarded: 0.0,
                                reason: rule.description.clone().unwrap_or_else(|| "Inference / enrichment action executed".to_string()),
                                phase: rule.phase.clone(),
                            });
                        }
                    } else if rule.is_eligibility_gate {
                        // Only treat failed condition as ineligibility if this is a positive prerequisite rule
                        let is_explicit_disqualification_rule = rule.actions.iter().any(|a| matches!(a, Action::SetEligibility { eligible: false, .. }));
                        if !is_explicit_disqualification_rule {
                            overall_eligible = false;
                            ineligibility_reasons.push(format!("Mandatory eligibility gate failed: {}", rule.name));
                        }
                    }
                }
            }
        }

        // Handle any remaining custom phases not in standard phase_order
        for (phase, mut rules) in rules_by_phase {
            if !phase_order.contains(&phase.as_str()) {
                rules.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.id.cmp(&b.id)));
                for rule in rules {
                    if let Some(ref group) = rule.activation_group {
                        if fired_activation_groups.contains(group) {
                            continue;
                        }
                    }
                    let passed = self.evaluate_condition(&ctx, &rule.condition)?;
                    if passed {
                        if let Some(ref group) = rule.activation_group {
                            fired_activation_groups.insert(group.clone());
                        }
                        let mut rule_awarded_points_or_eligibility = false;
                        for action in &rule.actions {
                            if matches!(action, Action::AwardPoints { .. } | Action::SetEligibility { .. }) {
                                rule_awarded_points_or_eligibility = true;
                            }
                            self.execute_action(
                                action,
                                rule,
                                &mut ctx,
                                &mut category_raw_scores,
                                &mut fired_rules,
                                &mut overall_eligible,
                                &mut ineligibility_reasons,
                            )?;
                        }
                        if !rule_awarded_points_or_eligibility {
                            fired_rules.push(FiredRuleRecord {
                                rule_id: rule.id.clone(),
                                rule_name: rule.name.clone(),
                                category: rule.category.clone(),
                                points_awarded: 0.0,
                                reason: rule.description.clone().unwrap_or_else(|| "Inference / enrichment action executed".to_string()),
                                phase: rule.phase.clone(),
                            });
                        }
                    } else if rule.is_eligibility_gate {
                        let is_explicit_disqualification_rule = rule.actions.iter().any(|a| matches!(a, Action::SetEligibility { eligible: false, .. }));
                        if !is_explicit_disqualification_rule {
                            overall_eligible = false;
                            ineligibility_reasons.push(format!("Mandatory eligibility gate failed: {}", rule.name));
                        }
                    }
                }
            }
        }

        // Apply Category Point Caps and Aggregate
        let mut category_scores = Vec::new();
        let mut total_score = 0.0;

        // Process defined categories
        for (cat_key, cat_cfg) in &self.program.categories {
            let raw_points = *category_raw_scores.get(cat_key).unwrap_or(&0.0);
            let (capped_points, is_capped) = match cat_cfg.max_points {
                Some(max) if raw_points > max => (max, true),
                _ => (raw_points, false),
            };

            category_scores.push(CategoryScoreBreakdown {
                category: cat_key.clone(),
                display_name: cat_cfg.display_name.clone(),
                raw_points,
                capped_points,
                max_points: cat_cfg.max_points,
                is_capped,
            });

            total_score += capped_points;
        }

        // Include any categories that earned points but weren't explicitly configured in program.categories
        for (cat_key, raw_points) in &category_raw_scores {
            if !self.program.categories.contains_key(cat_key) {
                category_scores.push(CategoryScoreBreakdown {
                    category: cat_key.clone(),
                    display_name: cat_key.clone(),
                    raw_points: *raw_points,
                    capped_points: *raw_points,
                    max_points: None,
                    is_capped: false,
                });
                total_score += *raw_points;
            }
        }

        // Apply Global Total Points Cap if defined
        if let Some(total_cap) = self.program.total_points_cap {
            if total_score > total_cap {
                total_score = total_cap;
            }
        }

        // Check Pass Mark Threshold
        let passed_threshold = match self.program.pass_mark_threshold {
            Some(threshold) => total_score >= threshold,
            None => true,
        };

        if !passed_threshold {
            ineligibility_reasons.push(format!(
                "Total score ({:.1}) is below the required pass mark threshold ({:.1})",
                total_score,
                self.program.pass_mark_threshold.unwrap_or(0.0)
            ));
        }

        Ok(AuditReport {
            program_id: self.program.id.clone(),
            program_name: self.program.name.clone(),
            version: self.program.version.clone(),
            applicant_id,
            eligible: overall_eligible,
            total_score,
            pass_mark_threshold: self.program.pass_mark_threshold,
            passed_threshold,
            category_scores,
            fired_rules,
            ineligibility_reasons,
            tags: ctx.get_tags().to_vec(),
        })
    }

    /// Evaluate an AST Condition against the FactContext
    pub fn evaluate_condition(&self, ctx: &FactContext, condition: &Condition) -> Result<bool> {
        match condition {
            Condition::Always => Ok(true),
            Condition::Exists { path } => Ok(ctx.path_exists(path)),
            Condition::All { conditions } => {
                for cond in conditions {
                    if !self.evaluate_condition(ctx, cond)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Any { conditions } => {
                for cond in conditions {
                    if self.evaluate_condition(ctx, cond)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::None { conditions } => {
                for cond in conditions {
                    if self.evaluate_condition(ctx, cond)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Not { condition } => {
                let res = self.evaluate_condition(ctx, condition)?;
                Ok(!res)
            }
            Condition::MinCount {
                min_required,
                conditions,
            } => {
                let mut passed = 0;
                for cond in conditions {
                    if self.evaluate_condition(ctx, cond)? {
                        passed += 1;
                    }
                }
                Ok(passed >= *min_required)
            }
            Condition::Temporal {
                field_path,
                window_days,
                ..
            } => {
                // If numeric test_age_days is available
                if let Ok(days) = ctx.get_i64(field_path) {
                    return Ok(days <= *window_days);
                }
                // Fallback presence check
                Ok(ctx.path_exists(field_path))
            }
            Condition::Compare { path, op, value } => {
                self.evaluate_comparison(ctx, path, op, value.as_ref())
            }
        }
    }

    /// Evaluate a single field comparison operator
    fn evaluate_comparison(
        &self,
        ctx: &FactContext,
        path: &str,
        op: &ComparisonOperator,
        target_value: Option<&Value>,
    ) -> Result<bool> {
        match op {
            ComparisonOperator::Exists => Ok(ctx.path_exists(path)),
            ComparisonOperator::DoesNotExist => Ok(!ctx.path_exists(path)),
            _ => {
                // If target path does not exist, comparisons evaluate to false (null-safe)
                let actual = match ctx.get_path(path) {
                    Ok(val) if !val.is_null() => val,
                    _ => return Ok(false),
                };

                let target = target_value.ok_or_else(|| {
                    EngineError::ConditionError(format!("Operator '{:?}' requires a target value at path '{path}'", op))
                })?;

                match op {
                    ComparisonOperator::Eq => Ok(values_equal(actual, target)),
                    ComparisonOperator::Neq => Ok(!values_equal(actual, target)),
                    ComparisonOperator::Gt => {
                        let a = value_to_f64(actual)?;
                        let t = value_to_f64(target)?;
                        Ok(a > t)
                    }
                    ComparisonOperator::Gte => {
                        let a = value_to_f64(actual)?;
                        let t = value_to_f64(target)?;
                        Ok(a >= t)
                    }
                    ComparisonOperator::Lt => {
                        let a = value_to_f64(actual)?;
                        let t = value_to_f64(target)?;
                        Ok(a < t)
                    }
                    ComparisonOperator::Lte => {
                        let a = value_to_f64(actual)?;
                        let t = value_to_f64(target)?;
                        Ok(a <= t)
                    }
                    ComparisonOperator::Between => {
                        let a = value_to_f64(actual)?;
                        if let Value::Array(bounds) = target {
                            if bounds.len() == 2 {
                                let min = value_to_f64(&bounds[0])?;
                                let max = value_to_f64(&bounds[1])?;
                                return Ok(a >= min && a <= max);
                            }
                        }
                        Err(EngineError::ConditionError(format!(
                            "Between operator requires an array [min, max] at '{path}'"
                        )))
                    }
                    ComparisonOperator::In => {
                        if let Value::Array(arr) = target {
                            for item in arr {
                                if values_equal(actual, item) {
                                    return Ok(true);
                                }
                            }
                            Ok(false)
                        } else {
                            Err(EngineError::ConditionError(format!(
                                "In operator requires an array at '{path}'"
                            )))
                        }
                    }
                    ComparisonOperator::NotIn => {
                        if let Value::Array(arr) = target {
                            for item in arr {
                                if values_equal(actual, item) {
                                    return Ok(false);
                                }
                            }
                            Ok(true)
                        } else {
                            Err(EngineError::ConditionError(format!(
                                "NotIn operator requires an array at '{path}'"
                            )))
                        }
                    }
                    ComparisonOperator::Contains => {
                        match actual {
                            Value::Array(arr) => {
                                for item in arr {
                                    if values_equal(item, target) {
                                        return Ok(true);
                                    }
                                }
                                Ok(false)
                            }
                            Value::String(s) => {
                                if let Value::String(sub) = target {
                                    Ok(s.contains(sub))
                                } else {
                                    Ok(false)
                                }
                            }
                            _ => Ok(false),
                        }
                    }
                    ComparisonOperator::MatchesRegex => {
                        if let (Some(s), Some(pattern)) = (actual.as_str(), target.as_str()) {
                            let re = Regex::new(pattern).map_err(|e| {
                                EngineError::ConditionError(format!("Invalid regex '{pattern}': {e}"))
                            })?;
                            Ok(re.is_match(s))
                        } else {
                            Ok(false)
                        }
                    }
                    _ => Ok(false),
                }
            }
        }
    }

    /// Calculate points based on formula
    pub fn calculate_points(&self, ctx: &FactContext, formula: &PointsFormula) -> Result<f64> {
        match formula {
            PointsFormula::Fixed { points } => Ok(*points),
            PointsFormula::Scaled {
                path,
                factor,
                min,
                max,
            } => {
                let val = ctx.get_f64(path).unwrap_or(0.0);
                let mut score = val * factor;
                if let Some(min_val) = min {
                    if score < *min_val {
                        score = *min_val;
                    }
                }
                if let Some(max_val) = max {
                    if score > *max_val {
                        score = *max_val;
                    }
                }
                Ok(score)
            }
            PointsFormula::Lookup {
                path,
                mapping,
                default,
            } => {
                let key = if let Ok(s) = ctx.get_str(path) {
                    s.to_string()
                } else if let Ok(i) = ctx.get_i64(path) {
                    i.to_string()
                } else {
                    return Ok(default.unwrap_or(0.0));
                };

                if let Some(points) = mapping.get(&key) {
                    Ok(*points)
                } else {
                    Ok(default.unwrap_or(0.0))
                }
            }
            PointsFormula::MatrixLookup {
                row_path,
                col_path,
                matrix,
                default,
            } => {
                let row_key = ctx.get_str(row_path).unwrap_or("");
                let col_key = ctx.get_str(col_path).unwrap_or("");

                if let Some(row) = matrix.get(row_key) {
                    if let Some(points) = row.get(col_key) {
                        return Ok(*points);
                    }
                }
                Ok(default.unwrap_or(0.0))
            }
            PointsFormula::LanguageClbBand {
                reading_path,
                writing_path,
                listening_path,
                speaking_path,
                band_scores,
            } => {
                let r = ctx.get_i64(reading_path).unwrap_or(0);
                let w = ctx.get_i64(writing_path).unwrap_or(0);
                let l = ctx.get_i64(listening_path).unwrap_or(0);
                let s = ctx.get_i64(speaking_path).unwrap_or(0);

                let lookup_band = |score: i64| -> f64 {
                    // Try exact match first
                    if let Some(pts) = band_scores.get(&score.to_string()) {
                        return *pts;
                    }
                    // Find highest matching threshold
                    let mut best_pts = 0.0;
                    for (band_str, pts) in band_scores {
                        if let Ok(band_num) = band_str.parse::<i64>() {
                            if score >= band_num && *pts > best_pts {
                                best_pts = *pts;
                            }
                        }
                    }
                    best_pts
                };

                let total = lookup_band(r) + lookup_band(w) + lookup_band(l) + lookup_band(s);
                Ok(total)
            }
            PointsFormula::DecisionTable(table) => self.evaluate_decision_table(ctx, table),
        }
    }

    /// Evaluate a multi-column Decision Table with DMN / Drools Hit Policies
    pub fn evaluate_decision_table(
        &self,
        ctx: &FactContext,
        table: &crate::core::ast::DecisionTable,
    ) -> Result<f64> {
        use crate::core::ast::HitPolicy;
        let mut matching_outputs: Vec<f64> = Vec::new();

        for row in &table.rows {
            let mut row_matches = true;
            for (idx, input_spec) in table.inputs.iter().enumerate() {
                if let Some(entry_pattern) = row.input_entries.get(idx) {
                    let pattern = entry_pattern.trim();
                    if pattern == "-" || pattern.is_empty() {
                        continue; // wildcard
                    }

                    let actual_val = ctx.get_path(&input_spec.path).ok();
                    if !match_decision_table_cell(actual_val, pattern) {
                        row_matches = false;
                        break;
                    }
                }
            }

            if row_matches {
                let out_val = row
                    .output_entries
                    .first()
                    .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
                    .unwrap_or(0.0);

                matching_outputs.push(out_val);

                if table.hit_policy == HitPolicy::First {
                    return Ok(out_val);
                }
            }
        }

        match table.hit_policy {
            HitPolicy::First => Ok(matching_outputs.first().copied().unwrap_or(0.0)),
            HitPolicy::Unique => {
                if matching_outputs.len() > 1 {
                    return Err(EngineError::EvaluationError(
                        "Decision table with Unique hit policy matched multiple rows".to_string(),
                    ));
                }
                Ok(matching_outputs.first().copied().unwrap_or(0.0))
            }
            HitPolicy::CollectSum => Ok(matching_outputs.iter().sum()),
            HitPolicy::CollectMax => {
                if matching_outputs.is_empty() {
                    Ok(0.0)
                } else {
                    Ok(matching_outputs.into_iter().fold(f64::NEG_INFINITY, f64::max))
                }
            }
            HitPolicy::CollectMin => {
                if matching_outputs.is_empty() {
                    Ok(0.0)
                } else {
                    Ok(matching_outputs.into_iter().fold(f64::INFINITY, f64::min))
                }
            }
            HitPolicy::CollectCount => Ok(matching_outputs.len() as f64),
            HitPolicy::Priority | HitPolicy::Any | HitPolicy::RuleOrder => {
                Ok(matching_outputs.first().copied().unwrap_or(0.0))
            }
        }
    }

    /// Execute a rule action
    fn execute_action(
        &self,
        action: &Action,
        rule: &Rule,
        ctx: &mut FactContext,
        category_raw_scores: &mut HashMap<String, f64>,
        fired_rules: &mut Vec<FiredRuleRecord>,
        overall_eligible: &mut bool,
        ineligibility_reasons: &mut Vec<String>,
    ) -> Result<()> {
        match action {
            Action::AwardPoints {
                category,
                formula,
                reason,
            } => {
                let points = self.calculate_points(ctx, formula)?;
                *category_raw_scores.entry(category.clone()).or_insert(0.0) += points;

                fired_rules.push(FiredRuleRecord {
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    category: category.clone(),
                    points_awarded: points,
                    reason: reason.clone(),
                    phase: rule.phase.clone(),
                });
            }
            Action::SetEligibility { eligible, reason } => {
                if !*eligible {
                    *overall_eligible = false;
                    ineligibility_reasons.push(format!("{}: {}", rule.name, reason));
                }
                fired_rules.push(FiredRuleRecord {
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    category: rule.category.clone(),
                    points_awarded: 0.0,
                    reason: reason.clone(),
                    phase: rule.phase.clone(),
                });
            }
            Action::AddTag { tag } => {
                ctx.add_tag(tag.clone());
            }
            Action::SetAttribute { key, value } => {
                ctx.set_computed_attribute(key.clone(), value.clone());
            }
            Action::RemoveAttribute { key } => {
                ctx.remove_computed_attribute(key);
            }
            Action::Accumulate {
                source_array,
                filter,
                function,
                field,
                target_attribute,
            } => {
                if let Ok(val) = ctx.get_path(source_array) {
                    if let Value::Array(items) = val {
                        let mut filtered_items = Vec::new();
                        for item in items {
                            if let Some(filt) = filter {
                                let item_ctx = FactContext::from_value(item.clone());
                                if self.evaluate_condition(&item_ctx, filt)? {
                                    filtered_items.push(item);
                                }
                            } else {
                                filtered_items.push(item);
                            }
                        }

                        let result_val = match function.as_str() {
                            "count" => Value::from(filtered_items.len()),
                            "sum" => {
                                let sum: f64 = filtered_items
                                    .iter()
                                    .filter_map(|i| {
                                        field.as_ref().and_then(|f| i.get(f)).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
                                    })
                                    .sum();
                                Value::from(sum)
                            }
                            "sum_fte_years" => {
                                // Normalized FTE calculation
                                let mut total_years = 0.0;
                                for item in filtered_items {
                                    let months = item.get("duration_months").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let hrs = item.get("weekly_hours").and_then(|v| v.as_f64()).unwrap_or(30.0);
                                    let fte_factor = if hrs >= 30.0 { 1.0 } else { hrs / 30.0 };
                                    total_years += (months * fte_factor) / 12.0;
                                }
                                Value::from(total_years)
                            }
                            _ => Value::Null,
                        };

                        ctx.set_computed_attribute(target_attribute.clone(), result_val);
                    }
                }
            }
        }
        Ok(())
    }
}

fn value_to_f64(v: &Value) -> Result<f64> {
    v.as_f64()
        .or_else(|| v.as_i64().map(|i| i as f64))
        .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
        .ok_or_else(|| EngineError::TypeMismatch {
            path: "".to_string(),
            expected: "numeric".to_string(),
            actual: format!("{:?}", v),
        })
}

fn values_equal(a: &Value, b: &Value) -> bool {
    if a == b {
        return true;
    }
    // String vs string comparison
    if let (Some(sa), Some(sb)) = (a.as_str(), b.as_str()) {
        return sa == sb;
    }
    // Numeric comparison (handle i64 vs f64)
    if let (Some(na), Some(nb)) = (a.as_f64().or_else(|| a.as_i64().map(|i| i as f64)), b.as_f64().or_else(|| b.as_i64().map(|i| i as f64))) {
        return (na - nb).abs() < f64::EPSILON;
    }
    // Boolean comparison
    if let (Some(ba), Some(bb)) = (a.as_bool(), b.as_bool()) {
        return ba == bb;
    }
    false
}

fn match_decision_table_cell(actual_val: Option<&Value>, pattern: &str) -> bool {
    let p = pattern.trim();
    if p.is_empty() || p == "-" || p == "*" {
        return true;
    }

    let actual = match actual_val {
        Some(v) if !v.is_null() => v,
        _ => return p.eq_ignore_ascii_case("null") || p.eq_ignore_ascii_case("none"),
    };

    // Range patterns: [min..max], [min, max], [min..max)
    if (p.starts_with('[') || p.starts_with('(')) && (p.ends_with(']') || p.ends_with(')')) {
        let inclusive_start = p.starts_with('[');
        let inclusive_end = p.ends_with(']');
        let inner = &p[1..p.len() - 1];
        let parts: Vec<&str> = if inner.contains("..") {
            inner.split("..").collect()
        } else if inner.contains(',') {
            inner.split(',').collect()
        } else {
            vec![]
        };

        if parts.len() == 2 {
            if let (Ok(min), Ok(max)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                if let Ok(act_num) = value_to_f64(actual) {
                    let start_ok = if inclusive_start { act_num >= min } else { act_num > min };
                    let end_ok = if inclusive_end { act_num <= max } else { act_num < max };
                    return start_ok && end_ok;
                }
            }
        }
    }

    // Comparison prefixes: >=, <=, >, <, !=, ==, =
    if let Some(target_str) = p.strip_prefix(">=") {
        if let (Ok(num), Ok(target)) = (value_to_f64(actual), target_str.trim().parse::<f64>()) {
            return num >= target;
        }
    } else if let Some(target_str) = p.strip_prefix("<=") {
        if let (Ok(num), Ok(target)) = (value_to_f64(actual), target_str.trim().parse::<f64>()) {
            return num <= target;
        }
    } else if let Some(target_str) = p.strip_prefix('>') {
        if let (Ok(num), Ok(target)) = (value_to_f64(actual), target_str.trim().parse::<f64>()) {
            return num > target;
        }
    } else if let Some(target_str) = p.strip_prefix('<') {
        if let (Ok(num), Ok(target)) = (value_to_f64(actual), target_str.trim().parse::<f64>()) {
            return num < target;
        }
    } else if let Some(target_str) = p.strip_prefix("!=") {
        let clean = target_str.trim().trim_matches('\'').trim_matches('"');
        if let Some(act_str) = actual.as_str() {
            return !act_str.eq_ignore_ascii_case(clean);
        } else if let Ok(target_num) = clean.parse::<f64>() {
            if let Ok(act_num) = value_to_f64(actual) {
                return (act_num - target_num).abs() > f64::EPSILON;
            }
        }
        return true;
    } else if p.starts_with("==") || p.starts_with('=') {
        let rest = if p.starts_with("==") { &p[2..] } else { &p[1..] };
        let target_str = rest.trim().trim_matches('\'').trim_matches('"');
        return match_exact(actual, target_str);
    }

    // In list: in ['a', 'b'], ['a', 'b']
    let in_list_str = if p.to_lowercase().starts_with("in ") {
        Some(p[3..].trim())
    } else if p.starts_with('[') && p.ends_with(']') {
        Some(p)
    } else {
        None
    };

    if let Some(list_expr) = in_list_str {
        let trimmed_list = list_expr.trim_start_matches('[').trim_end_matches(']');
        let items: Vec<&str> = trimmed_list
            .split(',')
            .map(|s| s.trim().trim_matches('\'').trim_matches('"'))
            .collect();
        for item in items {
            if match_exact(actual, item) {
                return true;
            }
        }
        return false;
    }

    // Default exact / literal match
    let clean_target = p.trim_matches('\'').trim_matches('"');
    match_exact(actual, clean_target)
}

fn match_exact(actual: &Value, target: &str) -> bool {
    if let Some(s) = actual.as_str() {
        return s.eq_ignore_ascii_case(target);
    }
    if let Some(b) = actual.as_bool() {
        if let Ok(tb) = target.parse::<bool>() {
            return b == tb;
        }
    }
    if let Ok(act_num) = value_to_f64(actual) {
        if let Ok(target_num) = target.parse::<f64>() {
            return (act_num - target_num).abs() < f64::EPSILON;
        }
    }
    false
}
