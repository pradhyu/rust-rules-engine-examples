use rust_rules_engine::{evaluate_facts, json_to_facts, load_knowledge_base_from_path};
use std::fs;
use std::path::Path;

fn load_program_and_context(rule_file: &str, applicant_file: &str) -> (rust_rules_engine::KnowledgeBase, Option<f64>, rust_rules_engine::Facts) {
    let rule_path = Path::new("rules").join(rule_file);
    let app_path = Path::new("applicants").join(applicant_file);

    let (kb, pass_mark) = load_knowledge_base_from_path(rule_path.to_str().unwrap()).unwrap();
    let app_content = fs::read_to_string(&app_path).unwrap();
    let app_val: serde_json::Value = serde_yaml::from_str(&app_content).unwrap();
    let facts = json_to_facts(&app_val);

    (kb, pass_mark, facts)
}

#[test]
fn test_tc01_skill_transferability_and_clb9() {
    let (kb, pass_mark, facts) = load_program_and_context(
        "canada_crs_express_entry.grl",
        "tc01_tech_lead_single.yaml",
    );
    let report = evaluate_facts(&kb, &facts, pass_mark).expect("Evaluation failed");

    assert!(report.is_eligible(), "Single Tech Lead should be eligible");
    assert!(
        report.total_score >= 100.0,
        "Total CRS score should be >= 100, got {}",
        report.total_score
    );
}

#[test]
fn test_tc04_multi_fact_relational_join_failure() {
    let (kb, pass_mark, facts) = load_program_and_context(
        "uk_skilled_worker_points.grl",
        "tc04_uk_sponsor_unlicensed.yaml",
    );
    let report = evaluate_facts(&kb, &facts, pass_mark).expect("Evaluation failed");

    assert!(
        !report.is_eligible(),
        "Candidate with revoked sponsor license must be ineligible"
    );
    assert_eq!(report.total_score, 50.0);
}

#[test]
fn test_tc05_activation_group_xor_mutual_exclusion() {
    let (kb, pass_mark, facts) = load_program_and_context(
        "uk_skilled_worker_points.grl",
        "tc05_uk_competing_tradeable.yaml",
    );
    let report = evaluate_facts(&kb, &facts, pass_mark).expect("Evaluation failed");

    assert!(report.is_eligible(), "Candidate should be eligible");
    assert_eq!(
        report.total_score, 70.0,
        "Total score should be exactly 70.0 (50 mandatory + 20 tradeable)"
    );

    let fired_names: Vec<&str> = report
        .fired_rules
        .iter()
        .map(|r| r.rule_name.as_str())
        .collect();
    assert!(
        fired_names.contains(&"uk_tradeable_option_a_salary"),
        "Option A must fire"
    );
    assert!(
        !fired_names.contains(&"uk_tradeable_option_b_stem_phd"),
        "Option B must be canceled by activation group"
    );
}

#[test]
fn test_tc08_three_valued_logic_null_safe_spouse() {
    let (kb, pass_mark, facts) = load_program_and_context(
        "canada_crs_express_entry.grl",
        "tc08_null_spouse_single.yaml",
    );
    let report = evaluate_facts(&kb, &facts, pass_mark).expect("Evaluation failed");

    // Must evaluate cleanly without crashing or panicking on null spouse
    assert!(report.is_eligible());
}

#[test]
fn test_tc11_forward_chaining_and_inferred_facts() {
    let (kb, pass_mark, facts) = load_program_and_context(
        "drools_parity_suite.grl",
        "tc11_forward_chaining.yaml",
    );
    let report = evaluate_facts(&kb, &facts, pass_mark).expect("Evaluation failed");

    assert!(report.is_eligible());
    assert!(!report.fired_rules.is_empty());
}
