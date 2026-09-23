use colored::Colorize;
use rust_rules_engine::{Engine, FactContext, RuleProgram};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🔄 MULTI-FORMAT RULE INGESTION & DSL DEMO (YAML, JSON, GRL) 🔄 "
            .bold()
            .on_cyan()
            .black()
    );
    println!(
        " Demonstrates loading and evaluating rules seamlessly across YAML, JSON, and GRL formats.\n"
    );

    // =========================================================================
    // 1. EVALUATION VIA YAML RULE PROGRAM
    // =========================================================================
    println!(
        "{}",
        "1️⃣  Evaluating via YAML Rules (rules/uk_skilled_worker_points.yaml)..."
            .bold()
            .underline()
    );
    let program_yaml = RuleProgram::from_yaml_file("rules/uk_skilled_worker_points.yaml")?;
    let engine_yaml = Engine::new(program_yaml);

    // Ingest YAML applicant
    let app_yaml_content = fs::read_to_string("applicants/tc05_uk_competing_tradeable.yaml")?;
    let app_val_from_yaml: serde_json::Value = serde_yaml::from_str(&app_yaml_content)?;
    let ctx_yaml = FactContext::from_value(app_val_from_yaml);

    let report_yaml = engine_yaml.evaluate(&ctx_yaml)?;
    println!(
        "   • Rule Set Loaded: {} (v{})",
        engine_yaml.program().name.cyan(),
        engine_yaml.program().version
    );
    println!(
        "   • Applicant Evaluated from: {}",
        "applicants/tc05_uk_competing_tradeable.yaml".yellow()
    );
    println!(
        "   • Result: Total Score = {:.1} / 70 (Status: {})\n",
        report_yaml.total_score,
        if report_yaml.is_eligible() {
            "Eligible".green()
        } else {
            "Ineligible".red()
        }
    );

    // =========================================================================
    // 2. EVALUATION VIA JSON RULE PROGRAM
    // =========================================================================
    println!(
        "{}",
        "2️⃣  Evaluating via JSON Rules (rules/uk_skilled_worker_points.json)..."
            .bold()
            .underline()
    );
    let json_content = fs::read_to_string("rules/uk_skilled_worker_points.json")?;
    let program_json = RuleProgram::from_json_str(&json_content)?;
    let engine_json = Engine::new(program_json);

    // Ingest JSON applicant
    let app_json_content = fs::read_to_string("applicants/tc01_tech_lead_single.json")?;
    let app_val_from_json: serde_json::Value = serde_json::from_str(&app_json_content)?;
    let ctx_json = FactContext::from_value(app_val_from_json);

    let report_json = engine_json.evaluate(&ctx_json)?;
    println!(
        "   • Rule Set Loaded: {} (v{})",
        engine_json.program().name.cyan(),
        engine_json.program().version
    );
    println!(
        "   • Applicant Evaluated from: {}",
        "applicants/tc01_tech_lead_single.json".yellow()
    );
    println!(
        "   • Result: Total Score = {:.1} (Status: {})\n",
        report_json.total_score,
        if report_json.is_eligible() {
            "Eligible".green()
        } else {
            "Ineligible".red()
        }
    );

    // =========================================================================
    // 3. GRL (GRULE RULE LANGUAGE) DSL PARITY PREVIEW
    // =========================================================================
    println!(
        "{}",
        "3️⃣  GRL (Grule Rule Language) Script Representation..."
            .bold()
            .underline()
    );
    let grl_sample = fs::read_to_string("rules/uk_skilled_worker_points.grl")?;
    let grl_lines: Vec<&str> = grl_sample.lines().take(22).collect();

    println!(
        "   Preview of '{}':",
        "rules/uk_skilled_worker_points.grl".yellow()
    );
    println!(
        "{}",
        "┌──────────────────────────────────────────────────────────────────────────┐".dimmed()
    );
    for line in grl_lines {
        println!("│ {:<72} │", line.dimmed());
    }
    println!(
        "│ {:<72} │",
        "... [remaining rules defined in file] ...".italic()
    );
    println!(
        "{}\n",
        "└──────────────────────────────────────────────────────────────────────────┘".dimmed()
    );

    println!("════════════════════════════════════════════════════════════════════════════");
    println!(" 🚀 Summary: In Rust, Serde enables 100% interoperability between YAML (on-disk),");
    println!("    JSON (REST APIs), and GRL DSL scripts compiling to the same unified AST.");
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
