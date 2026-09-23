use rust_rules_engine::{Engine, FactContext, RuleProgram};
use std::fs;
use std::path::Path;

fn load_program_and_context(rule_file: &str, applicant_file: &str) -> (RuleProgram, FactContext) {
    let rule_path = Path::new("rules").join(rule_file);
    let app_path = Path::new("applicants").join(applicant_file);

    let rule_content = fs::read_to_string(&rule_path)
        .unwrap_or_else(|e| panic!("Failed to read rule file {}: {}", rule_path.display(), e));
    let app_content = fs::read_to_string(&app_path)
        .unwrap_or_else(|e| panic!("Failed to read applicant file {}: {}", app_path.display(), e));

    let program = RuleProgram::from_yaml_str(&rule_content).expect("Failed to parse rule program YAML");
    let app_val: serde_json::Value = serde_yaml::from_str(&app_content).expect("Failed to parse applicant YAML");
    let context = FactContext::from_value(app_val);

    (program, context)
}

#[test]
fn test_tc01_skill_transferability_and_clb9() {
    let (program, context) = load_program_and_context("canada_crs_express_entry.yaml", "tc01_tech_lead_single.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    assert!(report.is_eligible(), "Single Tech Lead should be eligible");
    assert!(
        report.total_score >= 580.0,
        "Total CRS score should be >= 580, got {}",
        report.total_score
    );

    // Verify subcategory scores
    let transferability = report.category_scores.iter().find(|c| c.category == "skill_transferability").unwrap();
    assert_eq!(transferability.capped_points, 100.0, "Transferability should be at maximum cap of 100");

    let core = report.category_scores.iter().find(|c| c.category == "core_human_capital").unwrap();
    assert!(core.capped_points >= 400.0, "Core Human Capital should be >= 400");
}

#[test]
fn test_tc02_married_phd_provincial_nomination() {
    let (program, context) = load_program_and_context("canada_crs_express_entry.yaml", "tc02_married_phd_researcher.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    assert!(report.is_eligible());
    assert!(
        report.total_score >= 1000.0,
        "Total score with PNP should be >= 1000, got {}",
        report.total_score
    );

    let additional = report.category_scores.iter().find(|c| c.category == "additional_points").unwrap();
    assert_eq!(additional.capped_points, 600.0, "Provincial nomination should award 600 points");
}

#[test]
fn test_tc03_subcategory_cap_overflow_truncation() {
    let (program, context) = load_program_and_context("canada_crs_express_entry.yaml", "tc03_cap_overflow_tradesperson.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    assert!(report.is_eligible());
    let transferability = report.category_scores.iter().find(|c| c.category == "skill_transferability").unwrap();
    assert!(transferability.raw_points >= 150.0, "Raw transferability should be >= 150");
    assert_eq!(transferability.capped_points, 100.0, "Capped transferability must strictly be 100.0");
    assert!(transferability.is_capped, "Category should be flagged as is_capped = true");
}

#[test]
fn test_tc04_multi_fact_relational_join_failure() {
    let (program, context) = load_program_and_context("uk_skilled_worker_points.yaml", "tc04_uk_sponsor_unlicensed.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    assert!(!report.is_eligible(), "Candidate with revoked sponsor license must be ineligible");
    assert!(
        report.ineligibility_reasons.iter().any(|r| r.contains("Job Offer from Licensed Sponsor") || r.contains("sponsor")),
        "Ineligibility reasons must detail sponsor failure: {:?}",
        report.ineligibility_reasons
    );
}

#[test]
fn test_tc05_activation_group_xor_mutual_exclusion() {
    let (program, context) = load_program_and_context("uk_skilled_worker_points.yaml", "tc05_uk_competing_tradeable.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    assert!(report.is_eligible(), "Candidate should be eligible");
    assert_eq!(report.total_score, 70.0, "Total score should be exactly 70.0 (50 mandatory + 20 tradeable)");

    let tradeable = report.category_scores.iter().find(|c| c.category == "tradeable_criteria").unwrap();
    assert_eq!(tradeable.capped_points, 20.0, "Tradeable points should be exactly 20 (XOR mutual exclusion)");

    // Ensure Option A fired and Option B was skipped
    let fired_ids: Vec<&str> = report.fired_rules.iter().map(|r| r.rule_id.as_str()).collect();
    assert!(fired_ids.contains(&"uk_tradeable_option_a_salary"), "Option A must fire");
    assert!(!fired_ids.contains(&"uk_tradeable_option_b_stem_phd"), "Option B must be canceled by activation group");
}

#[test]
fn test_tc06_hard_age_gate_disqualification() {
    let (program, context) = load_program_and_context("australia_subclass_189.yaml", "tc06_australia_age_barred.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    assert!(!report.is_eligible(), "Age 46 candidate must be disqualified for Australia 189");
    assert!(report.ineligibility_reasons.iter().any(|r| r.contains("45 and over")), "Should contain age 45+ barrier reason");
}

#[test]
fn test_tc07_temporal_cep_expiration() {
    let (program, context) = load_program_and_context("edge_cases_drools_parity_suite.yaml", "tc07_temporal_expired_ielts.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    assert!(!report.is_eligible(), "Expired IELTS test (820 days old) must fail recency gate");
    assert!(report.ineligibility_reasons.iter().any(|r| r.contains("730 days")), "Should cite expiration limit");
}

#[test]
fn test_tc08_three_valued_logic_null_safe_spouse() {
    let (program, context) = load_program_and_context("edge_cases_drools_parity_suite.yaml", "tc08_null_spouse_single.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    // Must evaluate cleanly without crashing or panicking on null spouse
    assert!(report.is_eligible());
}

#[test]
fn test_tc10_forall_language_floor_failure() {
    let (program, context) = load_program_and_context("edge_cases_drools_parity_suite.yaml", "tc10_forall_language_fail.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    // Candidate has 10,10,10,6 -> rule requiring all 4 >= 7 must not fire
    let fired_ids: Vec<&str> = report.fired_rules.iter().map(|r| r.rule_id.as_str()).collect();
    assert!(!fired_ids.contains(&"ec_10_forall_language_floor"), "Universal quantifier rule must NOT fire");
}

#[test]
fn test_tc11_forward_chaining_and_inferred_facts() {
    let (program, context) = load_program_and_context("edge_cases_drools_parity_suite.yaml", "tc11_forward_chaining.yaml");
    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation failed");

    assert!(report.is_eligible());
    let fired_ids: Vec<&str> = report.fired_rules.iter().map(|r| r.rule_id.as_str()).collect();

    // Rule in Phase 1 inferring high language must have fired
    assert!(fired_ids.contains(&"ec_11_infer_clb9_proficiency"));
    // Downstream rule in Phase 3 relying on inferred fact must have fired
    assert!(fired_ids.contains(&"ec_11_downstream_transferability_fire"));
    assert!(report.tags.contains(&"inferred_clb9_mastery".to_string()));
}

#[test]
fn test_modular_directory_rule_loading() {
    // Load all modular rule files from the directory rules/canada_crs/
    let program = RuleProgram::from_path("rules/canada_crs").expect("Failed to load modular rule directory");
    assert_eq!(program.id, "canada_crs_express_entry");
    assert!(!program.categories.is_empty(), "Categories should be merged from directory files");
    assert!(program.rules.len() >= 10, "Rules should be merged from all modular files in directory");

    let app_content = std::fs::read_to_string("applicants/tc01_tech_lead_single.yaml").expect("Failed to read applicant");
    let app_val: serde_json::Value = serde_yaml::from_str(&app_content).expect("Failed to parse applicant");
    let context = FactContext::from_value(app_val);

    let engine = Engine::new(program);
    let report = engine.evaluate(&context).expect("Evaluation from modular directory failed");

    assert!(report.is_eligible());
    assert!(report.total_score >= 580.0, "Score should match single file evaluation (got {})", report.total_score);
}

#[test]
fn test_decision_table_hit_policies() {
    use rust_rules_engine::{DecisionTable, DecisionTableInput, DecisionTableOutput, DecisionTableRow, HitPolicy};
    use serde_json::json;

    let mut ctx_val = serde_json::Map::new();
    ctx_val.insert("age".to_string(), json!(28));
    ctx_val.insert("education_level".to_string(), json!("master"));
    ctx_val.insert("clb_score".to_string(), json!(9));
    let ctx = FactContext::from_value(serde_json::Value::Object(ctx_val));

    // Build a sample multi-column decision table:
    // Inputs: age, education_level, clb_score
    // Row 1: [20..29], master, >= 9  -> 50 pts
    // Row 2: [20..29], -, >= 7       -> 25 pts
    // Row 3: >= 30, -, -             -> 10 pts
    let inputs = vec![
        DecisionTableInput { name: "age".to_string(), path: "age".to_string() },
        DecisionTableInput { name: "education".to_string(), path: "education_level".to_string() },
        DecisionTableInput { name: "clb".to_string(), path: "clb_score".to_string() },
    ];
    let outputs = vec![DecisionTableOutput { name: "points".to_string(), category: "skills".to_string() }];
    let rows = vec![
        DecisionTableRow {
            id: Some("r1".to_string()),
            description: Some("Young Master CLB9".to_string()),
            input_entries: vec!["[20..29]".to_string(), "in ['master', 'doctorate']".to_string(), ">= 9".to_string()],
            output_entries: vec![json!(50.0)],
        },
        DecisionTableRow {
            id: Some("r2".to_string()),
            description: Some("Young Good CLB".to_string()),
            input_entries: vec!["[20..29]".to_string(), "-".to_string(), ">= 7".to_string()],
            output_entries: vec![json!(25.0)],
        },
        DecisionTableRow {
            id: Some("r3".to_string()),
            description: Some("Older candidate".to_string()),
            input_entries: vec![">= 30".to_string(), "-".to_string(), "-".to_string()],
            output_entries: vec![json!(10.0)],
        },
    ];

    let engine = Engine::new(RuleProgram {
        id: "dt_test".to_string(),
        name: "Decision Table Test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        categories: std::collections::HashMap::new(),
        total_points_cap: None,
        pass_mark_threshold: None,
        rules: vec![],
    });

    // 1. CollectSum (both Row 1 and Row 2 match -> 50 + 25 = 75)
    let mut dt_collect_sum = DecisionTable {
        id: "dt_1".to_string(),
        name: "Skills Table".to_string(),
        hit_policy: HitPolicy::CollectSum,
        inputs: inputs.clone(),
        outputs: outputs.clone(),
        rows: rows.clone(),
    };
    let score_sum = engine.evaluate_decision_table(&ctx, &dt_collect_sum).expect("CollectSum failed");
    assert_eq!(score_sum, 75.0, "CollectSum should sum matched rows 50 + 25 = 75");

    // 2. First (first matching row is Row 1 -> 50)
    dt_collect_sum.hit_policy = HitPolicy::First;
    let score_first = engine.evaluate_decision_table(&ctx, &dt_collect_sum).expect("First failed");
    assert_eq!(score_first, 50.0, "First hit policy should return first matching row (50)");

    // 3. CollectMax (max between 50 and 25 is 50)
    dt_collect_sum.hit_policy = HitPolicy::CollectMax;
    let score_max = engine.evaluate_decision_table(&ctx, &dt_collect_sum).expect("CollectMax failed");
    assert_eq!(score_max, 50.0);

    // 4. CollectMin (min between 50 and 25 is 25)
    dt_collect_sum.hit_policy = HitPolicy::CollectMin;
    let score_min = engine.evaluate_decision_table(&ctx, &dt_collect_sum).expect("CollectMin failed");
    assert_eq!(score_min, 25.0);

    // 5. CollectCount (2 rows matched)
    dt_collect_sum.hit_policy = HitPolicy::CollectCount;
    let score_count = engine.evaluate_decision_table(&ctx, &dt_collect_sum).expect("CollectCount failed");
    assert_eq!(score_count, 2.0);

    // 6. Unique hit policy with multiple matches must return an error
    dt_collect_sum.hit_policy = HitPolicy::Unique;
    assert!(engine.evaluate_decision_table(&ctx, &dt_collect_sum).is_err(), "Unique must fail on multiple matches");
}

#[test]
fn test_decision_table_csv_spreadsheet_parsing() {
    use rust_rules_engine::DecisionTable;
    use serde_json::json;

    let csv_content = r#"
# Drools / DMN Spreadsheet Decision Table Example
age, education_level, points, description
[18..35], master, 50, Prime age with Masters
[18..35], bachelor, 30, Prime age with Bachelors
>= 36, master, 35, Mid career Masters
>= 36, bachelor, 20, Mid career Bachelors
"#;

    let dt = DecisionTable::from_csv_str(csv_content).expect("Failed to parse CSV Decision Table");
    assert_eq!(dt.inputs.len(), 2);
    assert_eq!(dt.rows.len(), 4);

    let mut ctx_val = serde_json::Map::new();
    ctx_val.insert("age".to_string(), json!(29));
    ctx_val.insert("education_level".to_string(), json!("master"));
    let ctx = FactContext::from_value(serde_json::Value::Object(ctx_val));

    let engine = Engine::new(RuleProgram {
        id: "csv_test".to_string(),
        name: "CSV Decision Table Test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        categories: std::collections::HashMap::new(),
        total_points_cap: None,
        pass_mark_threshold: None,
        rules: vec![],
    });

    let score = engine.evaluate_decision_table(&ctx, &dt).expect("CSV evaluation failed");
    assert_eq!(score, 50.0, "Prime age Master should match 50 points");
}

#[test]
fn test_decision_table_integration_in_rule_program() {
    use rust_rules_engine::{Action, Condition, DecisionTable, HitPolicy, PointsFormula, Rule};
    use serde_json::json;

    let csv_content = r#"
canadian_work_years, foreign_work_years, points, description
>= 2, >= 3, 50, High Canadian and Foreign Experience
>= 1, >= 1, 25, Moderate Experience
"#;

    let dt = DecisionTable::from_csv_str(csv_content).expect("Failed to parse CSV");
    let mut dt_rule = dt;
    dt_rule.hit_policy = HitPolicy::First;

    let rule = Rule {
        id: "dt_rule_experience".to_string(),
        name: "Experience Decision Matrix".to_string(),
        description: Some("Award points via Decision Table".to_string()),
        phase: "scoring".to_string(),
        category: "skill_transferability".to_string(),
        priority: 100,
        activation_group: None,
        no_loop: false,
        is_eligibility_gate: false,
        enabled: true,
        condition: Condition::Always,
        actions: vec![Action::AwardPoints {
            category: "skill_transferability".to_string(),
            formula: PointsFormula::DecisionTable(Box::new(dt_rule)),
            reason: "Evaluated via Decision Table".to_string(),
        }],
    };

    let program = RuleProgram {
        id: "dt_program".to_string(),
        name: "Decision Table Program".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        categories: std::collections::HashMap::new(),
        total_points_cap: None,
        pass_mark_threshold: None,
        rules: vec![rule],
    };

    let mut ctx_val = serde_json::Map::new();
    ctx_val.insert("canadian_work_years".to_string(), json!(3));
    ctx_val.insert("foreign_work_years".to_string(), json!(4));
    let ctx = FactContext::from_value(serde_json::Value::Object(ctx_val));

    let engine = Engine::new(program);
    let report = engine.evaluate(&ctx).expect("Rule Program with Decision Table failed");

    assert_eq!(report.total_score, 50.0);
    assert_eq!(report.fired_rules.len(), 1);
    assert_eq!(report.fired_rules[0].points_awarded, 50.0);
}

