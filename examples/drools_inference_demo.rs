use rust_rules_engine::{evaluate_facts, json_to_facts, load_knowledge_base_from_path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Drools-Equivalent Forward Chaining & Inference Demo (via KSD-CO/rust-rule-engine) ⚡\n");

    // 1. Load the GRL parity rule suite
    let (kb, pass_mark) = load_knowledge_base_from_path("rules/drools_parity_suite.grl")?;

    // 2. Load candidate testing forward chaining
    let app_content = std::fs::read_to_string("applicants/tc11_forward_chaining.yaml")?;
    let app_val: serde_json::Value = serde_yaml::from_str(&app_content)?;
    let facts = json_to_facts(&app_val);

    // 3. Evaluate: Phase 1 derives facts, downstream rules fire in subsequent cycles
    let report = evaluate_facts(&kb, &facts, pass_mark)?;

    println!("Candidate ID: {:?}", report.applicant_id);
    println!("Inferred Tags: {:?}", report.tags);
    println!("Total Points Earned: {:.1}", report.total_score);

    // Print itemized audit trail
    report.print_audit_table();

    Ok(())
}
