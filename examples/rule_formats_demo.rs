use colored::Colorize;
use rust_rules_engine::{
    Facts, RuleEngineBuilder, Value, evaluate_facts, json_to_facts, load_knowledge_base_from_path,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🔄 MULTI-FORMAT RULE INGESTION & DSL DEMO (GRL, JSON, YAML via KSD-CO/rust-rule-engine) 🔄 "
            .bold()
            .on_cyan()
            .black()
    );
    println!(
        " Demonstrates loading GRL specifications and evaluating applicant facts formatted as YAML and JSON.\n"
    );

    // =========================================================================
    // 1. EVALUATION VIA GRL RULE FILE WITH YAML APPLICANT
    // =========================================================================
    println!(
        "{}",
        "1️⃣  Evaluating GRL Rules with YAML Applicant Data..."
            .bold()
            .underline()
    );
    let (kb, pass_mark) = load_knowledge_base_from_path("rules/uk_skilled_worker_points.grl")?;

    let app_yaml_content = fs::read_to_string("applicants/tc05_uk_competing_tradeable.yaml")?;
    let app_val_from_yaml: serde_json::Value = serde_yaml::from_str(&app_yaml_content)?;
    let facts_yaml = json_to_facts(&app_val_from_yaml);

    let report_yaml = evaluate_facts(&kb, &facts_yaml, pass_mark)?;
    println!("   • Knowledge Base: {}", kb.name().cyan());
    println!("   • Applicant Source: applicants/tc05_uk_competing_tradeable.yaml (YAML)");
    println!(
        "   • Result: Total Score = {:.1} (Status: {})\n",
        report_yaml.total_score,
        if report_yaml.is_eligible() {
            "Eligible".green()
        } else {
            "Ineligible".red()
        }
    );

    // =========================================================================
    // 2. EVALUATION VIA GRL RULE FILE WITH JSON APPLICANT
    // =========================================================================
    println!(
        "{}",
        "2️⃣  Evaluating GRL Rules with JSON Applicant Data..."
            .bold()
            .underline()
    );
    let (kb_crs, pass_mark_crs) = load_knowledge_base_from_path("rules/canada_crs_express_entry.grl")?;

    let app_json_content = fs::read_to_string("applicants/tc01_tech_lead_single.json")?;
    let app_val_from_json: serde_json::Value = serde_json::from_str(&app_json_content)?;
    let facts_json = json_to_facts(&app_val_from_json);

    let report_json = evaluate_facts(&kb_crs, &facts_json, pass_mark_crs)?;
    println!("   • Knowledge Base: {}", kb_crs.name().cyan());
    println!("   • Applicant Source: applicants/tc01_tech_lead_single.json (JSON)");
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
    // 3. EVALUATION VIA INLINE GRL USING RuleEngineBuilder
    // =========================================================================
    println!(
        "{}",
        "3️⃣  Evaluating Inline GRL via RuleEngineBuilder..."
            .bold()
            .underline()
    );
    let inline_grl = r#"
        rule "ExpeditedTechVisa" salience 100 {
            when
                applicant.skills.years_experience >= 5 &&
                applicant.skills.is_tech == true
            then
                applicant.fast_track = true;
                applicant.priority_tier = "Tier1";
        }
    "#;

    let mut engine = RuleEngineBuilder::new()
        .with_inline_grl(inline_grl)?
        .build();

    let facts = Facts::new();
    let applicant_data = serde_json::json!({
        "skills": {
            "years_experience": 8,
            "is_tech": true
        }
    });
    facts.add_value("applicant", Value::from(applicant_data))?;

    let result = engine.execute(&facts)?;
    println!("   • Inline GRL rules evaluated: {}", result.rules_evaluated);
    println!("   • Rules fired: {}", result.rules_fired);
    println!(
        "   • Inferred applicant.fast_track: {:?}",
        facts.get_nested("applicant.fast_track")
    );
    println!(
        "   • Inferred applicant.priority_tier: {:?}\n",
        facts.get_nested("applicant.priority_tier")
    );

    println!("{}", "✅ Multi-Format Rule Ingestion Verification Complete!".green().bold());
    Ok(())
}
