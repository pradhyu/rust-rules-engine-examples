use rust_rule_engine::{GRLParser, Facts, Value};
use std::fs;

#[test]
fn test_parse_simple_grl() {
    let grl = r#"
    rule "CheckAge" salience 10 {
        when
            User.Age >= 18 && User.Country == "US"
        then
            User.IsAdult = true;
            Log("User is an adult");
    }
    "#;

    let rules = GRLParser::parse_rules(grl).expect("Failed to parse basic GRL");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "CheckAge");
    assert_eq!(rules[0].salience, 10);
}

#[test]
fn test_parse_uk_skilled_worker_grl() {
    let grl_content = fs::read_to_string("rules/uk_skilled_worker_points.grl").expect("File read failed");
    match GRLParser::parse_rules(&grl_content) {
        Ok(rules) => {
            println!("Successfully parsed {} rules from uk_skilled_worker_points.grl", rules.len());
            for r in &rules {
                println!("  Rule: {} (salience {})", r.name, r.salience);
                println!("    Actions: {:?}", r.actions);
            }
        }
        Err(e) => {
            panic!("Parse error: {}", e);
        }
    }
}

#[test]
fn test_execute_uk_skilled_worker_rules() {
    use rust_rule_engine::{KnowledgeBase, RustRuleEngine};
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::Arc;

    let grl_content = fs::read_to_string("rules/uk_skilled_worker_points.grl").expect("File read failed");
    let rules = GRLParser::parse_rules(&grl_content).expect("Failed to parse GRL");

    let kb = KnowledgeBase::new("UK_Skilled_Worker");
    for mut rule in rules {
        rule.no_loop = true;
        kb.add_rule(rule).expect("Failed to add rule");
    }

    let mut config = rust_rule_engine::EngineConfig::default();
    config.max_cycles = 1;
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Track points awarded
    let total_points = Arc::new(AtomicI64::new(0));
    let points_clone = Arc::clone(&total_points);

    engine.register_action_handler("award_points", move |params, _facts| {
        let category = params.get("0").and_then(|v| v.as_string()).unwrap_or_default();
        let pts = params.get("1").and_then(|v| v.as_number()).unwrap_or(0.0) as i64;
        println!("  -> Custom Action Handler: award_points: {} = +{} pts", category, pts);
        points_clone.fetch_add(pts, Ordering::SeqCst);
        Ok(())
    });

    // Populate facts
    let facts = Facts::new();
    let app_json = r#"{
        "job_offer": {
            "has_offer": true,
            "sponsor": {
                "is_licensed": true,
                "license_rating": "A_rated"
            },
            "rqf_skill_level": 4,
            "annual_salary": 42000.0,
            "meets_occupation_going_rate": true
        },
        "language": {
            "cefr_level": "B2"
        },
        "education": {
            "highest_degree": "master",
            "is_stem": true
        }
    }"#;

    let parsed_json: serde_json::Value = serde_json::from_str(app_json).unwrap();
    facts.add_value("applicant", Value::from(parsed_json)).unwrap();

    let result = engine.execute(&facts).expect("Engine execution failed");
    println!("Execution result: {} rules fired in {} cycles", result.rules_fired, result.cycle_count);
    let final_pts = total_points.load(Ordering::SeqCst);
    println!("Total Points Earned: {}", final_pts);

    assert_eq!(result.rules_fired, 4); // sponsor (20), rqf (20), english (10), option A salary (20)
    assert_eq!(final_pts, 70); // 20 + 20 + 10 + 20 = 70 points! Pass mark!
}

#[test]
fn test_parse_canada_crs_grl() {
    let grl_content = fs::read_to_string("rules/canada_crs_express_entry.grl").expect("File read failed");
    let rules = GRLParser::parse_rules(&grl_content).expect("Failed to parse Canada CRS GRL");
    println!("Successfully parsed {} rules from canada_crs_express_entry.grl", rules.len());
    assert!(rules.len() >= 9);
}



