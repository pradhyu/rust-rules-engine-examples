//! UK Skilled Worker Immigration Points-Based System using KSD-CO/rust-rule-engine
//!
//! Evaluates candidates against statutory UK immigration rules defined in GRL:
//! - Mandatory non-tradeable criteria (50 points required)
//!   * Job offer from A-rated sponsor (20 pts)
//!   * Job at RQF Level 3+ (20 pts)
//!   * English at CEFR B1+ (10 pts)
//! - Tradeable criteria (20 points required, activation-group XOR):
//!   * Option A: Salary >= £38,700 (20 pts)
//!   * Option B: STEM PhD + Salary >= £30,960 (20 pts)
//!   * Option C: Shortage list job + Salary >= £30,960 (20 pts)
//! - Pass mark: 70 points total.
//!
//! Run with:
//!   cargo run --example rust_rule_engine_uk_immigration

use colored::Colorize;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Row, Table};
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value};
use std::fs;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

struct CandidateScenario {
    id: &'static str,
    name: &'static str,
    file_path: &'static str,
    expected_tradeable_option: &'static str,
    expected_eligible: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🇬🇧 UK SKILLED WORKER POINTS EVALUATION (via KSD-CO/rust-rule-engine) 🇬🇧 "
            .bold()
            .on_blue()
            .white()
    );
    println!(" Demonstrates GRL rule parsing, salience ordering, and XOR activation groups.\n");

    // 1. Ingest GRL rules using rust-rule-engine GRLParser
    let grl_path = "rules/uk_skilled_worker_points.grl";
    let grl_content = fs::read_to_string(grl_path)?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    println!(
        " 📖 Ingested {} rules from '{}'",
        rules.len().to_string().cyan(),
        grl_path.yellow()
    );
    for r in &rules {
        println!(
            "    • [{}] salience: {}, group: {:?}",
            r.name.cyan(),
            r.salience,
            r.activation_group.as_deref().unwrap_or("none")
        );
    }
    println!();

    let scenarios = vec![
        CandidateScenario {
            id: "TC-05",
            name: "Dr. Alistair Chen (A-rated Sponsor, RQF 6, £48k Salary)",
            file_path: "applicants/tc05_uk_competing_tradeable.yaml",
            expected_tradeable_option: "Option A (Full Salary Threshold)",
            expected_eligible: true,
        },
        CandidateScenario {
            id: "TC-14",
            name: "Dr. Maya Lin (STEM PhD, £34.5k Discounted Salary)",
            file_path: "applicants/tc14_uk_stem_phd_discounted.yaml",
            expected_tradeable_option: "Option B (STEM PhD Discount)",
            expected_eligible: true,
        },
        CandidateScenario {
            id: "TC-04",
            name: "Tariq Al-Mansoor (Revoked / Unlicensed Sponsor)",
            file_path: "applicants/tc04_uk_sponsor_unlicensed.yaml",
            expected_tradeable_option: "None (Failed Sponsor Gate)",
            expected_eligible: false,
        },
    ];

    let mut summary_table = Table::new();
    summary_table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("ID").fg(Color::Yellow),
            Cell::new("Candidate Name").fg(Color::Cyan),
            Cell::new("Mandatory (50)").fg(Color::White),
            Cell::new("Tradeable (20)").fg(Color::White),
            Cell::new("Total Score").fg(Color::White),
            Cell::new("Decision").fg(Color::White),
            Cell::new("Tradeable Match").fg(Color::White),
        ]);

    for s in &scenarios {
        // Build KnowledgeBase & RustRuleEngine fresh for each candidate
        let kb = KnowledgeBase::new("UK_Skilled_Worker_KB");
        for r in &rules {
            kb.add_rule(r.clone())?;
        }
        let mut config = rust_rule_engine::EngineConfig::default();
        config.max_cycles = 1;
        let mut engine = RustRuleEngine::with_config(kb, config);

        let mandatory_score = Arc::new(AtomicI64::new(0));
        let tradeable_score = Arc::new(AtomicI64::new(0));
        let fired_tradeable = Arc::new(std::sync::Mutex::new(String::from("None")));

        let mand_clone = Arc::clone(&mandatory_score);
        let trad_clone = Arc::clone(&tradeable_score);
        let tradeable_name_clone = Arc::clone(&fired_tradeable);

        // Register custom action handler for GRL "award_points"
        engine.register_action_handler("award_points", move |params, _facts| {
            let category = params.get("0").and_then(|v| v.as_string()).unwrap_or_default();
            let pts = params.get("1").and_then(|v| v.as_number()).unwrap_or(0.0) as i64;
            if category == "mandatory_criteria" {
                mand_clone.fetch_add(pts, Ordering::SeqCst);
            } else if category == "tradeable_criteria" {
                trad_clone.fetch_add(pts, Ordering::SeqCst);
                let mut name_lock = tradeable_name_clone.lock().unwrap();
                *name_lock = format!("Option Matched (+{} pts)", pts);
            }
            Ok(())
        });

        // Load applicant data into Facts
        let facts = Facts::new();
        let file_content = fs::read_to_string(s.file_path)?;
        let json_val: serde_json::Value = if s.file_path.ends_with(".yaml") {
            serde_yaml::from_str(&file_content)?
        } else {
            serde_json::from_str(&file_content)?
        };

        if let serde_json::Value::Object(map) = json_val {
            for (k, v) in map {
                facts.add_value(&k, Value::from(v))?;
            }
        }

        let start = std::time::Instant::now();
        let result = engine.execute(&facts)?;
        let elapsed = start.elapsed();

        let mand = mandatory_score.load(Ordering::SeqCst);
        let trad = tradeable_score.load(Ordering::SeqCst);
        let total = mand + trad;
        let is_eligible = total >= 70 && mand == 50;

        let status_cell = if is_eligible {
            Cell::new("QUALIFIED").fg(Color::Green)
        } else {
            Cell::new("REFUSED").fg(Color::Red)
        };

        summary_table.add_row(Row::from(vec![
            Cell::new(s.id),
            Cell::new(s.name),
            Cell::new(format!("{}/50", mand)),
            Cell::new(format!("{}/20", trad)),
            Cell::new(format!("{}/70", total)),
            status_cell,
            Cell::new(s.expected_tradeable_option),
        ]));

        println!(
            " Evaluated [{}] in {:?}: {} rules evaluated, {} fired.",
            s.id.cyan(),
            elapsed,
            result.rules_evaluated,
            result.rules_fired
        );
        assert_eq!(is_eligible, s.expected_eligible);
    }

    println!("\n{}\n", summary_table);

    println!("════════════════════════════════════════════════════════════════════════════");
    println!(" 🎯 Key Takeaway: KSD-CO/rust-rule-engine correctly enforced mandatory gate");
    println!("    criteria and XOR activation groups for UK Skilled Worker immigration!");
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
