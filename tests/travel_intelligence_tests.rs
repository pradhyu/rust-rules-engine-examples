use rust_rule_engine::{Facts, RuleEngineBuilder, Value};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::Instant;

const TRAVEL_RULES_GRL: &str = r#"
rule "EnrichSanctionedVisits" salience 900 {
    when
        applicant.sanctioned_count > 0
    then
        applicant.compliance_risk = "high";
        applicant.security_review = true;
}

rule "EnrichHighOverstays" salience 900 {
    when
        applicant.overstay_count > 3
    then
        applicant.compliance_risk = "high";
}

rule "EnrichCleanCompliance" salience 880 {
    when
        applicant.overstay_count == 0 && applicant.sanctioned_count == 0
    then
        applicant.compliance_risk = "low";
}

rule "FiveEyesTrusted" salience 850 {
    when
        applicant.five_eyes_count >= 5
    then
        applicant.five_eyes_trusted = true;
}

rule "SchengenOverstayRisk" salience 800 {
    when
        applicant.schengen_days > 180
    then
        applicant.schengen_risk = true;
}

rule "DecisionAutoDeny" salience 500 activation-group "decision" {
    when
        applicant.compliance_risk == "high" && applicant.schengen_risk == true
    then
        applicant.eligible = false;
        applicant.decision = "denied";
        applicant.reason = "High compliance risk with Schengen overstay";
}

rule "DecisionAutoApprove" salience 490 activation-group "decision" {
    when
        applicant.compliance_risk == "low" && applicant.five_eyes_trusted == true
    then
        applicant.eligible = true;
        applicant.decision = "auto_approve";
        applicant.tag_fast_track = true;
        applicant.total_score = applicant.total_score + 100.0;
}

rule "DecisionManualReview" salience 480 activation-group "decision" {
    when
        applicant.compliance_risk == "moderate" || applicant.security_review == true
    then
        applicant.eligible = true;
        applicant.decision = "manual_review";
        applicant.total_score = applicant.total_score + 30.0;
}

rule "VisaTierPremium" salience 300 activation-group "visa_tier" {
    when
        applicant.decision == "auto_approve" && applicant.total_trips >= 50
    then
        applicant.visa_tier = "10-Year Multiple Entry";
        applicant.total_score = applicant.total_score + 50.0;
}
"#;

fn evaluate_applicant_file(applicant_file: &str) -> (Facts, rust_rule_engine::GruleExecutionResult) {
    let app_path = Path::new("applicants").join(applicant_file);
    let app_content = fs::read_to_string(&app_path).unwrap();
    let app_val: serde_json::Value = serde_yaml::from_str(&app_content).unwrap();

    let history = app_val["applicant"]["travel_history"].as_array().unwrap();
    let mut fe_count = 0;
    let mut overstay_count = 0;
    let mut sanctioned_count = 0;
    let mut conflict_count = 0;
    let mut schengen_days = 0;

    for r in history {
        let cc = r["country_code"].as_str().unwrap_or("");
        let reg = r["region"].as_str().unwrap_or("");
        let over = r["overstay_days"].as_i64().unwrap_or(0);
        let dur = r["duration_days"].as_i64().unwrap_or(0);

        if reg == "five_eyes" {
            fe_count += 1;
        }
        if over > 0 {
            overstay_count += 1;
        }
        if ["IR", "KP", "SY", "CU", "VE"].contains(&cc) {
            sanctioned_count += 1;
        }
        if ["AF", "IQ", "LY", "SO", "YE", "UA"].contains(&cc) {
            conflict_count += 1;
        }
        if reg == "schengen" {
            schengen_days += dur;
        }
    }

    let facts = Facts::new();
    let applicant_obj = json!({
        "id": app_val["applicant"]["id"],
        "name": app_val["applicant"]["name"],
        "nationality": app_val["applicant"]["nationality"],
        "total_trips": history.len() as i64,
        "five_eyes_count": fe_count,
        "overstay_count": overstay_count,
        "sanctioned_count": sanctioned_count,
        "conflict_count": conflict_count,
        "schengen_days": schengen_days,
        "five_eyes_trusted": false,
        "security_review": conflict_count > 0,
        "schengen_risk": schengen_days > 180,
        "compliance_risk": "unknown",
        "eligible": true,
        "decision": "pending",
        "visa_tier": "pending",
        "tag_fast_track": false,
        "total_score": 50.0
    });

    facts.add_value("applicant", Value::from(applicant_obj)).unwrap();

    let mut engine = RuleEngineBuilder::new()
        .with_inline_grl(TRAVEL_RULES_GRL)
        .unwrap()
        .build();

    let result = engine.execute(&facts).unwrap();
    (facts, result)
}

#[test]
fn test_tc12_frequent_business_traveler_auto_approve() {
    let (facts, result) = evaluate_applicant_file("tc12_frequent_business_traveler.yaml");

    assert!(result.rules_fired > 0);
    let eligible = facts.get_nested("applicant.eligible").and_then(|v| v.as_boolean()).unwrap();
    let decision = facts.get_nested("applicant.decision").and_then(|v| v.as_string()).unwrap();
    let score = facts.get_nested("applicant.total_score").and_then(|v| v.as_number()).unwrap();
    let fast_track = facts.get_nested("applicant.tag_fast_track").and_then(|v| v.as_boolean()).unwrap();

    assert!(eligible);
    assert_eq!(decision, "auto_approve");
    assert!(score >= 67.0);
    assert!(fast_track);
}

#[test]
fn test_tc13_high_risk_traveler_denied() {
    let (facts, _) = evaluate_applicant_file("tc13_high_risk_traveler.yaml");

    let eligible = facts.get_nested("applicant.eligible").and_then(|v| v.as_boolean()).unwrap();
    let decision = facts.get_nested("applicant.decision").and_then(|v| v.as_string()).unwrap();

    assert!(!eligible);
    assert_eq!(decision, "denied");
}

#[test]
fn test_travel_large_dataset_performance() {
    let facts = Facts::new();
    let applicant_obj = json!({
        "id": "PERF-001",
        "name": "High Volume Traveler",
        "total_trips": 2000,
        "five_eyes_count": 1500,
        "overstay_count": 0,
        "sanctioned_count": 0,
        "conflict_count": 0,
        "schengen_days": 40,
        "five_eyes_trusted": false,
        "security_review": false,
        "schengen_risk": false,
        "compliance_risk": "unknown",
        "eligible": true,
        "decision": "pending",
        "visa_tier": "pending",
        "tag_fast_track": false,
        "total_score": 50.0
    });

    facts.add_value("applicant", Value::from(applicant_obj)).unwrap();

    let mut engine = RuleEngineBuilder::new()
        .with_inline_grl(TRAVEL_RULES_GRL)
        .unwrap()
        .build();

    let start = Instant::now();
    let result = engine.execute(&facts).unwrap();
    let elapsed = start.elapsed();

    println!("Performance test completed in: {:?}", elapsed);
    assert!(result.rules_fired >= 3);
    let eligible = facts.get_nested("applicant.eligible").and_then(|v| v.as_boolean()).unwrap();
    assert!(eligible);
}
