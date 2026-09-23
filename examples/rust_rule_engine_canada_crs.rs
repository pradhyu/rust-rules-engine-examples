//! Canada Comprehensive Ranking System (CRS) Express Entry Demo via KSD-CO/rust-rule-engine
//!
//! Demonstrates:
//! - Ingesting Canada Express Entry rules from `rules/canada_crs_express_entry.grl`
//! - Multi-phase forward-chaining inference across execution cycles:
//!   * Cycle 1: Language scores (CLB 9 in all 4 abilities) derive `has_clb9_mastery`
//!   * Cycle 2: Skill transferability rules activate using the newly inferred fact
//! - Category breakdown: Core Human Capital, Skill Transferability, Additional Factors
//!
//! Run with:
//!   cargo run --example rust_rule_engine_canada_crs

use colored::Colorize;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Row, Table};
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🍁 CANADA CRS EXPRESS ENTRY INFERENCE (via KSD-CO/rust-rule-engine) 🍁 "
            .bold()
            .on_red()
            .white()
    );
    println!(" Demonstrates multi-cycle forward chaining, fact inference, and score aggregation.\n");

    // 1. Ingest GRL rules
    let grl_path = "rules/canada_crs_express_entry.grl";
    let grl_content = fs::read_to_string(grl_path)?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    println!(
        " 📖 Ingested {} rules from '{}'",
        rules.len().to_string().cyan(),
        grl_path.yellow()
    );

    let kb = KnowledgeBase::new("Canada_CRS_KB");
    for r in &rules {
        kb.add_rule(r.clone())?;
    }

    let mut engine = RustRuleEngine::new(kb);

    // Track category points and tags
    let category_points: Arc<Mutex<HashMap<String, f64>>> = Arc::new(Mutex::new(HashMap::new()));
    let tags: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let pts_clone = Arc::clone(&category_points);
    engine.register_action_handler("award_points", move |params, _facts| {
        let category = params.get("0").and_then(|v| v.as_string()).unwrap_or_default();
        let pts = params.get("1").and_then(|v| v.as_number()).unwrap_or(0.0);
        let mut map = pts_clone.lock().unwrap();
        *map.entry(category).or_insert(0.0) += pts;
        Ok(())
    });

    let tags_clone = Arc::clone(&tags);
    engine.register_action_handler("award_tag", move |params, _facts| {
        let tag = params.get("0").and_then(|v| v.as_string()).unwrap_or_default();
        let mut list = tags_clone.lock().unwrap();
        list.push(tag);
        Ok(())
    });

    // Load applicant
    let app_path = "applicants/tc01_tech_lead_single.json";
    let app_content = fs::read_to_string(app_path)?;
    let app_json: serde_json::Value = serde_json::from_str(&app_content)?;

    let facts = Facts::new();
    if let serde_json::Value::Object(map) = app_json {
        for (k, v) in map {
            facts.add_value(&k, Value::from(v))?;
        }
    }

    println!(" 👤 Loaded Applicant: Liam Tremblay (Age 29, Single, Master's degree, CLB 9 Language, 2yr Canadian + 3yr Foreign Experience)");

    // Execute forward chaining
    let start = std::time::Instant::now();
    let result = engine.execute(&facts)?;
    let elapsed = start.elapsed();

    println!(
        "\n ⚡ Execution Completed in {:?}: {} rules evaluated, {} fired across {} cycles.\n",
        elapsed,
        result.rules_evaluated,
        result.rules_fired,
        result.cycle_count
    );

    // Display Category Breakdown
    let map = category_points.lock().unwrap();
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("CRS Factor Category").fg(Color::Yellow),
            Cell::new("Points Awarded").fg(Color::Cyan),
            Cell::new("Statutory Cap").fg(Color::White),
        ]);

    let core_pts = *map.get("core_human_capital").unwrap_or(&0.0);
    let skill_pts = *map.get("skill_transferability").unwrap_or(&0.0);
    let bonus_pts = *map.get("additional_factors").unwrap_or(&0.0);
    let total_crs = core_pts + skill_pts + bonus_pts;

    table.add_row(Row::from(vec![
        Cell::new("1. Core Human Capital (Age 110 + Master's 135 + Cdn Work 53)"),
        Cell::new(format!("{:.1}", core_pts)).fg(Color::Green),
        Cell::new("500 max"),
    ]));
    table.add_row(Row::from(vec![
        Cell::new("2. Skill Transferability (Master's+CLB9 50 + Foreign+CLB9 50 + Foreign+Cdn 50)"),
        Cell::new(format!("{:.1} (capped at 100)", skill_pts.min(100.0))).fg(Color::Green),
        Cell::new("100 max"),
    ]));
    table.add_row(Row::from(vec![
        Cell::new("3. Additional Factors (Sibling in Canada PR/Citizen)"),
        Cell::new(format!("{:.1}", bonus_pts)).fg(Color::Green),
        Cell::new("600 max"),
    ]));
    table.add_row(Row::from(vec![
        Cell::new("TOTAL COMPREHENSIVE RANKING SYSTEM (CRS) SCORE").fg(Color::Yellow),
        Cell::new(format!("{:.1} / 1200", total_crs)).fg(Color::Yellow),
        Cell::new("1200 max"),
    ]));

    println!("{}\n", table);

    let assigned_tags = tags.lock().unwrap();
    println!(" 🏷️ Assigned Tags: {:?}", assigned_tags);

    println!("\n════════════════════════════════════════════════════════════════════════════");
    println!(" 🎯 Key Takeaway: Phase 1 rules inferred CLB 9 language mastery, which");
    println!("    subsequently triggered Phase 3 Skill Transferability in cycle 2!");
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
