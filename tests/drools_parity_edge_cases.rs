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
