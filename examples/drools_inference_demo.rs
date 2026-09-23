use rust_rules_engine::{Engine, FactContext, RuleProgram};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Drools-Equivalent Forward Chaining & Inference Demo ⚡\n");

    // 1. Load the edge cases rule suite
    let program = RuleProgram::from_yaml_file("rules/edge_cases_drools_parity_suite.yaml")?;
    let engine = Engine::new(program);

    // 2. Load candidate testing forward chaining
    let app_content = std::fs::read_to_string("applicants/tc11_forward_chaining.yaml")?;
    let app_val: serde_json::Value = serde_yaml::from_str(&app_content)?;
    let context = FactContext::from_value(app_val);

    // 3. Evaluate: Phase 1 derives high language proficiency, Phase 3 fires downstream rule
    let report = engine.evaluate(&context)?;

    println!("Candidate ID: {:?}", report.applicant_id);
    println!("Inferred Tags: {:?}", report.tags);
    println!("Total Points Earned: {:.1}", report.total_score);

    // Print itemized audit trail
    report.print_audit_table();

    Ok(())
}
