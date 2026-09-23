use colored::Colorize;
use rust_rule_engine::{Facts, RuleEngineBuilder, Value};
use serde_json::json;
use std::time::Instant;

struct TravelerConfig {
    id: &'static str,
    name: &'static str,
    nationality: &'static str,
    total_trips: usize,
    overstays: usize,
    sanctioned: usize,
    conflict: usize,
    schengen_days: usize,
    mainly_five_eyes: bool,
}

fn generate_travel_history(config: &TravelerConfig) -> Vec<serde_json::Value> {
    let mut history = Vec::new();
    let mut remaining_trips = config.total_trips;

    for _ in 0..config.overstays {
        history.push(json!({
            "country_code": "FR",
            "region": "schengen",
            "entry_date": "2023-01-01",
            "exit_date": "2023-01-20",
            "duration_days": 19,
            "purpose": "tourism",
            "visa_type": "C",
            "overstay_days": 5
        }));
        remaining_trips -= 1;
    }

    for _ in 0..config.sanctioned {
        history.push(json!({
            "country_code": "IR",
            "region": "other",
            "entry_date": "2023-02-01",
            "exit_date": "2023-02-10",
            "duration_days": 9,
            "purpose": "business",
            "visa_type": "B",
            "overstay_days": 0
        }));
        remaining_trips -= 1;
    }

    for _ in 0..config.conflict {
        history.push(json!({
            "country_code": "AF",
            "region": "other",
            "entry_date": "2023-03-01",
            "exit_date": "2023-03-05",
            "duration_days": 4,
            "purpose": "ngo",
            "visa_type": "V",
            "overstay_days": 0
        }));
        remaining_trips -= 1;
    }

    if config.schengen_days > 0 {
        history.push(json!({
            "country_code": "DE",
            "region": "schengen",
            "entry_date": "2023-06-01",
            "exit_date": "2023-06-30",
            "duration_days": config.schengen_days,
            "purpose": "tourism",
            "visa_type": "C",
            "overstay_days": 0
        }));
        if remaining_trips > 0 {
            remaining_trips -= 1;
        }
    }

    if config.mainly_five_eyes {
        let fe_countries = ["US", "GB", "CA", "AU", "NZ"];
        for i in 0..remaining_trips {
            let cc = fe_countries[i % fe_countries.len()];
            history.push(json!({
                "country_code": cc,
                "region": "five_eyes",
                "entry_date": "2023-04-01",
                "exit_date": "2023-04-08",
                "duration_days": 7,
                "purpose": "business",
                "visa_type": "B1",
                "overstay_days": 0
            }));
        }
    } else {
        let other_countries = ["DE", "IT", "ES", "NL", "JP", "SG", "BR", "ZA"];
        for i in 0..remaining_trips {
            let cc = other_countries[i % other_countries.len()];
            let region = if ["DE", "IT", "ES", "NL"].contains(&cc) {
                "schengen"
            } else {
                "other"
            };
            history.push(json!({
                "country_code": cc,
                "region": region,
                "entry_date": "2023-05-01",
                "exit_date": "2023-05-15",
                "duration_days": 14,
                "purpose": "tourism",
                "visa_type": "C",
                "overstay_days": 0
            }));
        }
    }

    history
}

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

rule "EnrichModerateOverstays" salience 890 {
    when
        applicant.overstay_count >= 1 && applicant.overstay_count <= 3 && applicant.sanctioned_count == 0
    then
        applicant.compliance_risk = "moderate";
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
        applicant.decision == "auto_approve" && applicant.total_trips >= 100
    then
        applicant.visa_tier = "10-Year Multiple Entry";
        applicant.total_score = applicant.total_score + 50.0;
}

rule "VisaTierStandard" salience 290 activation-group "visa_tier" {
    when
        applicant.decision == "auto_approve" && applicant.total_trips < 100
    then
        applicant.visa_tier = "5-Year Standard";
        applicant.total_score = applicant.total_score + 25.0;
}

rule "VisaTierRestricted" salience 280 activation-group "visa_tier" {
    when
        applicant.decision == "manual_review"
    then
        applicant.visa_tier = "1-Year Single Entry (Interview Required)";
        applicant.total_score = applicant.total_score + 10.0;
}
"#;

fn evaluate_scenario(config: &TravelerConfig) {
    println!("\n========================================================");
    println!("👤 Scenario: {} ({})", config.name.bold(), config.id.cyan());
    println!("========================================================");

    let history = generate_travel_history(config);
    println!("🧳 Generated {} travel records.", history.len());

    // Aggregate summary features from records
    let mut fe_count = 0;
    let mut overstay_count = 0;
    let mut sanctioned_count = 0;
    let mut conflict_count = 0;
    let mut schengen_days = 0;

    for r in &history {
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
        "id": config.id,
        "name": config.name,
        "nationality": config.nationality,
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
        "total_score": 50.0
    });

    facts.add_value("applicant", Value::from(applicant_obj)).unwrap();

    let mut engine = RuleEngineBuilder::new()
        .with_inline_grl(TRAVEL_RULES_GRL)
        .unwrap()
        .build();

    let start = Instant::now();
    let exec_res = engine.execute(&facts).unwrap();
    let duration = start.elapsed();

    let eligible = facts.get_nested("applicant.eligible").and_then(|v| v.as_boolean()).unwrap_or(true);
    let decision = facts.get_nested("applicant.decision").and_then(|v| v.as_string()).unwrap_or_default();
    let visa_tier = facts.get_nested("applicant.visa_tier").and_then(|v| v.as_string()).unwrap_or_default();
    let total_score = facts.get_nested("applicant.total_score").and_then(|v| v.as_number()).unwrap_or(0.0);
    let compliance_risk = facts.get_nested("applicant.compliance_risk").and_then(|v| v.as_string()).unwrap_or_default();

    println!("⏱️  Processing time: {:?}", duration);
    println!("⚡ Rules Evaluated: {}, Fired: {}", exec_res.rules_evaluated, exec_res.rules_fired);
    println!("🔍 Compliance Risk Level: {}", compliance_risk.yellow().bold());
    println!("🎯 Final Decision:       {}", decision.cyan().bold());
    println!("🏆 Total Points:          {:.1}", total_score);
    println!("🛂 Visa Tier Assigned:   {}", visa_tier.green().bold());
    println!(
        "✅ Qualification Status:  {}",
        if eligible && decision != "denied" {
            "APPROVED / QUALIFIED".green().bold()
        } else {
            "DENIED / DISQUALIFIED".red().bold()
        }
    );
}

fn main() {
    println!(
        "\n{}",
        " 🌍 GLOBAL TRAVEL INTELLIGENCE VISA ASSESSMENT (via KSD-CO/rust-rule-engine) 🌍 "
            .bold()
            .on_blue()
            .white()
    );
    println!(" Multi-layer inferencing over high-volume historical travel records.\n");

    let scenario_a = TravelerConfig {
        id: "TRAV-001",
        name: "Frequent Business Traveler",
        nationality: "JP",
        total_trips: 1200,
        overstays: 0,
        sanctioned: 0,
        conflict: 0,
        schengen_days: 30,
        mainly_five_eyes: true,
    };

    let scenario_b = TravelerConfig {
        id: "TRAV-002",
        name: "Moderate Risk Traveler",
        nationality: "BR",
        total_trips: 800,
        overstays: 2,
        sanctioned: 0,
        conflict: 1,
        schengen_days: 60,
        mainly_five_eyes: true,
    };

    let scenario_c = TravelerConfig {
        id: "TRAV-003",
        name: "High Risk Traveler",
        nationality: "XX",
        total_trips: 500,
        overstays: 5,
        sanctioned: 3,
        conflict: 0,
        schengen_days: 210,
        mainly_five_eyes: false,
    };

    evaluate_scenario(&scenario_a);
    evaluate_scenario(&scenario_b);
    evaluate_scenario(&scenario_c);
}
