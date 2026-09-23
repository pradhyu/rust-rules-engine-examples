use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use rust_rule_engine::engine::engine::EngineConfig;
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvaluatorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Rule engine error: {0}")]
    RuleEngine(#[from] rust_rule_engine::RuleEngineError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Error: {0}")]
    Custom(String),
}

impl From<String> for EvaluatorError {
    fn from(s: String) -> Self {
        EvaluatorError::Custom(s)
    }
}

impl From<&str> for EvaluatorError {
    fn from(s: &str) -> Self {
        EvaluatorError::Custom(s.to_string())
    }
}

/// Category Configuration Metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryConfig {
    pub name: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_points: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Category Score Breakdown
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryScoreBreakdown {
    pub category: String,
    pub display_name: String,
    pub raw_points: f64,
    pub capped_points: f64,
    pub max_points: Option<f64>,
    pub is_capped: bool,
}

/// Record of an executed rule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FiredRuleRecord {
    pub rule_id: String,
    pub rule_name: String,
    pub category: String,
    pub points_awarded: f64,
    pub reason: String,
    pub phase: String,
}

/// Comprehensive audit report of an evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub program_id: String,
    pub program_name: String,
    pub version: String,
    pub applicant_id: Option<String>,
    pub eligible: bool,
    pub total_score: f64,
    pub pass_mark_threshold: Option<f64>,
    pub passed_threshold: bool,
    pub category_scores: Vec<CategoryScoreBreakdown>,
    pub fired_rules: Vec<FiredRuleRecord>,
    pub ineligibility_reasons: Vec<String>,
    pub tags: Vec<String>,
}

impl AuditReport {
    pub fn is_eligible(&self) -> bool {
        self.eligible && self.passed_threshold
    }

    /// Print a formatted audit table to stdout
    pub fn print_audit_table(&self) {
        println!();
        let title = format!(
            " ⚖️  DECISION AUDIT REPORT: {} ({}) ",
            self.program_name, self.version
        );
        println!("{}", title.bold().on_blue().white());
        if let Some(ref id) = self.applicant_id {
            println!("  Applicant ID: {}", id.cyan().bold());
        }

        let status_str = if self.is_eligible() {
            " QUALIFIED / ELIGIBLE ".bold().on_green().white()
        } else {
            " NOT QUALIFIED / DISQUALIFIED ".bold().on_red().white()
        };
        println!("  Overall Result: {status_str}");
        println!(
            "  Total Calculated Score: {} points",
            format!("{:.1}", self.total_score).yellow().bold()
        );

        if let Some(threshold) = self.pass_mark_threshold {
            println!(
                "  Pass Mark Threshold:    {:.1} points (Passed: {})",
                threshold,
                if self.passed_threshold {
                    "YES".green().bold()
                } else {
                    "NO".red().bold()
                }
            );
        }

        if !self.ineligibility_reasons.is_empty() {
            println!("\n{}", "❌ Ineligibility / Gate Failures:".bold().red());
            for reason in &self.ineligibility_reasons {
                println!("  • {}", reason.red());
            }
        }

        if !self.category_scores.is_empty() {
            println!("\n{}", "📊 Category Score Breakdown:".bold().underline());
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Category").fg(Color::Cyan),
                    Cell::new("Raw Points").fg(Color::Yellow),
                    Cell::new("Cap Limit").fg(Color::White),
                    Cell::new("Capped Points").fg(Color::Green),
                    Cell::new("Status").fg(Color::White),
                ]);

            for cat in &self.category_scores {
                let cap_str = cat
                    .max_points
                    .map(|m| format!("{:.1}", m))
                    .unwrap_or_else(|| "None".to_string());
                let status = if cat.is_capped {
                    "Capped".yellow()
                } else {
                    "OK".green()
                };

                table.add_row(vec![
                    Cell::new(&cat.display_name),
                    Cell::new(format!("{:.1}", cat.raw_points)),
                    Cell::new(cap_str),
                    Cell::new(format!("{:.1}", cat.capped_points)).fg(Color::Green),
                    Cell::new(status.to_string()),
                ]);
            }
            println!("{table}");
        }

        if !self.fired_rules.is_empty() {
            println!("\n{}", "📋 Fired Rules Audit Trace:".bold().underline());
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Rule ID").fg(Color::Cyan),
                    Cell::new("Rule Name").fg(Color::White),
                    Cell::new("Category").fg(Color::Yellow),
                    Cell::new("Points").fg(Color::Green),
                ]);

            for r in &self.fired_rules {
                let pts_str = if r.points_awarded > 0.0 {
                    format!("+{:.1}", r.points_awarded)
                } else {
                    "—".to_string()
                };
                table.add_row(vec![
                    Cell::new(&r.rule_id),
                    Cell::new(&r.rule_name),
                    Cell::new(&r.category),
                    Cell::new(pts_str).fg(Color::Green),
                ]);
            }
            println!("{table}");
        }

        if !self.tags.is_empty() {
            println!("\n{}", "🏷️ Inferred Tags:".bold());
            let tags_str = self
                .tags
                .iter()
                .map(|t| format!("[{}]", t.cyan()))
                .collect::<Vec<_>>()
                .join(" ");
            println!("  {tags_str}");
        }
        println!();
    }
}

/// Convert a serde_json::Value into rust_rule_engine Facts
pub fn json_to_facts(val: &serde_json::Value) -> Facts {
    let facts = Facts::new();
    if let serde_json::Value::Object(map) = val {
        if let Some(app) = map.get("applicant") {
            let _ = facts.add_value("applicant", Value::from(app.clone()));
        } else {
            let _ = facts.add_value("applicant", Value::from(serde_json::Value::Object(map.clone())));
        }
        for (k, v) in map {
            let _ = facts.add_value(k, Value::from(v.clone()));
        }
    }
    facts
}

/// Load a KnowledgeBase and optional pass mark threshold from a GRL file or directory
pub fn load_knowledge_base_from_path(
    path_str: &str,
) -> Result<(KnowledgeBase, Option<f64>), EvaluatorError> {
    let path = Path::new(path_str);
    let mut grl_files = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_file() && p.extension().map_or(false, |ext| ext == "grl") {
                grl_files.push(p);
            }
        }
        grl_files.sort();
    } else if path.is_file() {
        grl_files.push(path.to_path_buf());
    } else {
        return Err(format!("Rules path not found: {}", path_str).into());
    }

    if grl_files.is_empty() {
        return Err(format!("No .grl rule files found in '{}'", path_str).into());
    }

    let kb_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("DefaultKB");
    let kb = KnowledgeBase::new(kb_name);
    let mut pass_mark = None;

    for file in grl_files {
        let content = fs::read_to_string(&file)?;
        // Inspect header comments for pass mark
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("// @pass_mark:") {
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() >= 2 {
                    if let Ok(m) = parts[1].trim().parse::<f64>() {
                        pass_mark = Some(m);
                    }
                }
            }
        }

        let rules = GRLParser::parse_rules(&content)
            .map_err(|e| format!("Error parsing GRL '{}': {}", file.display(), e))?;
        for r in rules {
            kb.add_rule(r)?;
        }
    }

    Ok((kb, pass_mark))
}

/// Evaluate facts against a KnowledgeBase using RustRuleEngine
pub fn evaluate_facts(
    kb: &KnowledgeBase,
    facts: &Facts,
    pass_mark_threshold: Option<f64>,
) -> Result<AuditReport, EvaluatorError> {
    let has_activation_group = kb.get_rules().iter().any(|r| r.activation_group.is_some());
    let mut config = EngineConfig::default();
    config.max_cycles = if has_activation_group { 1 } else { 10 };
    let mut engine = RustRuleEngine::with_config(kb.clone(), config);

    let category_points: Arc<Mutex<HashMap<String, f64>>> = Arc::new(Mutex::new(HashMap::new()));
    let fired_records: Arc<Mutex<Vec<FiredRuleRecord>>> = Arc::new(Mutex::new(Vec::new()));
    let tags: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let disqualifications: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Custom action: award_points(category, points)
    let cat_pts_clone = Arc::clone(&category_points);
    let fired_clone = Arc::clone(&fired_records);
    engine.register_action_handler("award_points", move |params, _facts| {
        let category = params.get("0").and_then(|v| v.as_string()).unwrap_or_else(|| "general".to_string());
        let pts = params.get("1").and_then(|v| v.as_number()).unwrap_or(0.0);
        *cat_pts_clone.lock().unwrap().entry(category.clone()).or_insert(0.0) += pts;
        fired_clone.lock().unwrap().push(FiredRuleRecord {
            rule_id: format!("award_{}", category),
            rule_name: format!("Award points to {}", category),
            category,
            points_awarded: pts,
            reason: format!("Awarded {:.1} points", pts),
            phase: "scoring".to_string(),
        });
        Ok(())
    });

    // Custom action: award_tag(tag)
    let tags_clone = Arc::clone(&tags);
    engine.register_action_handler("award_tag", move |params, _facts| {
        let tag = params.get("0").and_then(|v| v.as_string()).unwrap_or_default();
        if !tag.is_empty() {
            tags_clone.lock().unwrap().push(tag);
        }
        Ok(())
    });

    // Custom action: disqualify(reason)
    let disq_clone = Arc::clone(&disqualifications);
    engine.register_action_handler("disqualify", move |params, _facts| {
        let reason = params.get("0").and_then(|v| v.as_string()).unwrap_or_else(|| "Disqualified".to_string());
        disq_clone.lock().unwrap().push(reason);
        Ok(())
    });

    // Custom action: Log(message)
    engine.register_action_handler("Log", move |_params, _facts| Ok(()));

    // Execute with fired rule callback
    let fired_names: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let names_clone = Arc::clone(&fired_names);
    let _exec_result = engine.execute_with_callback(facts, move |rule_name, _facts| {
        names_clone.lock().unwrap().push(rule_name.to_string());
    })?;

    // Combine fired rules
    let mut fired_rules = fired_records.lock().unwrap().clone();
    for name in fired_names.lock().unwrap().iter() {
        if !fired_rules.iter().any(|r| &r.rule_name == name) {
            fired_rules.push(FiredRuleRecord {
                rule_id: name.clone(),
                rule_name: name.clone(),
                category: "rule_engine".to_string(),
                points_awarded: 0.0,
                reason: format!("Rule '{}' condition evaluated true", name),
                phase: "evaluation".to_string(),
            });
        }
    }

    // Read scores
    let cat_map = category_points.lock().unwrap().clone();
    let mut total_score = 0.0;
    let mut category_scores = Vec::new();

    for (cat_name, raw_pts) in cat_map {
        total_score += raw_pts;
        category_scores.push(CategoryScoreBreakdown {
            category: cat_name.clone(),
            display_name: cat_name.replace('_', " "),
            raw_points: raw_pts,
            capped_points: raw_pts,
            max_points: None,
            is_capped: false,
        });
    }

    // Check if total_score was set directly in facts
    if let Some(fact_score) = facts
        .get_nested("applicant.total_score")
        .or_else(|| facts.get("total_score"))
    {
        if let Some(num) = fact_score.as_number() {
            if num > 0.0 && total_score == 0.0 {
                total_score = num;
            }
        }
    }

    let mut ineligibility_reasons = disqualifications.lock().unwrap().clone();
    let passed_threshold = match pass_mark_threshold {
        Some(threshold) => {
            if total_score < threshold {
                ineligibility_reasons.push(format!(
                    "Total score ({:.1}) is below the required pass mark threshold ({:.1})",
                    total_score, threshold
                ));
                false
            } else {
                true
            }
        }
        None => true,
    };

    let eligible = ineligibility_reasons.is_empty();

    let applicant_id = facts
        .get_nested("applicant.id")
        .or_else(|| facts.get("id"))
        .and_then(|v| v.as_string().map(|s| s.to_string()));

    Ok(AuditReport {
        program_id: kb.name().to_string(),
        program_name: kb.name().to_string(),
        version: format!("v{}", kb.version()),
        applicant_id,
        eligible,
        total_score,
        pass_mark_threshold,
        passed_threshold,
        category_scores,
        fired_rules,
        ineligibility_reasons,
        tags: tags.lock().unwrap().clone(),
    })
}
