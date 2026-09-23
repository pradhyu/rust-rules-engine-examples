use rust_rules_engine::{evaluate_facts, json_to_facts, load_knowledge_base_from_path};
use std::fs;

#[test]
fn test_load_uk_skilled_worker_grl() {
    let (kb, pass_mark) = load_knowledge_base_from_path("rules/uk_skilled_worker_points.grl")
        .expect("Failed to load and parse uk_skilled_worker_points.grl");

    assert_eq!(kb.name(), "uk_skilled_worker_points");
    assert_eq!(pass_mark, Some(70.0));
    assert_eq!(kb.rule_count(), 6);
}

#[test]
fn test_evaluate_uk_skilled_worker_grl() {
    let (kb, pass_mark) = load_knowledge_base_from_path("rules/uk_skilled_worker_points.grl").unwrap();

    let app_content = fs::read_to_string("applicants/tc05_uk_competing_tradeable.yaml").unwrap();
    let app_val: serde_json::Value = serde_yaml::from_str(&app_content).unwrap();
    let facts = json_to_facts(&app_val);

    let report = evaluate_facts(&kb, &facts, pass_mark).unwrap();

    assert_eq!(report.total_score, 70.0);
    assert!(report.is_eligible());
    assert!(!report.fired_rules.is_empty());
}

#[test]
fn test_evaluate_uk_skilled_worker_ineligible() {
    let (kb, pass_mark) = load_knowledge_base_from_path("rules/uk_skilled_worker_points.grl").unwrap();

    let app_content = fs::read_to_string("applicants/tc04_uk_sponsor_unlicensed.yaml").unwrap();
    let app_val: serde_json::Value = serde_yaml::from_str(&app_content).unwrap();
    let facts = json_to_facts(&app_val);

    let report = evaluate_facts(&kb, &facts, pass_mark).unwrap();

    // With unlicensed sponsor, mandatory criterion fails, total is 50.0 < 70.0 pass mark
    assert!(!report.is_eligible());
    assert_eq!(report.total_score, 50.0);
}

#[test]
fn test_load_canada_crs_grl() {
    let (kb, _) = load_knowledge_base_from_path("rules/canada_crs_express_entry.grl")
        .expect("Failed to load and parse canada_crs_express_entry.grl");

    assert_eq!(kb.name(), "canada_crs_express_entry");
    assert!(kb.rule_count() >= 10);
}

#[test]
fn test_load_drools_parity_suite_grl() {
    let (kb, _) = load_knowledge_base_from_path("rules/drools_parity_suite.grl")
        .expect("Failed to load and parse drools_parity_suite.grl");

    assert!(kb.rule_count() > 0);
}
