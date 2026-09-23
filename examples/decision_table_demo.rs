use rust_rules_engine::{
    Action, Condition, DecisionTable, Engine, FactContext, HitPolicy, PointsFormula, Rule,
    RuleProgram,
};
use serde_json::json;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Drools & DMN Decision Table Evaluation Demo 📊\n");

    // 1. Parse CSV spreadsheet decision table
    let csv_content = fs::read_to_string("rules/sample_decision_table.csv")?;
    let mut decision_table = DecisionTable::from_csv_str(&csv_content)?;
    decision_table.hit_policy = HitPolicy::First; // DMN Hit Policy: First matching row

    println!("Loaded Decision Table: '{}' with {} rows", decision_table.name, decision_table.rows.len());
    println!("Hit Policy: {:?}", decision_table.hit_policy);
    println!("Input Columns: {:?}", decision_table.inputs.iter().map(|i| &i.name).collect::<Vec<_>>());

    // 2. Build a Rule that delegates scoring to the Decision Table
    let dt_rule = Rule {
        id: "crs_dt_foreign_work_education".to_string(),
        name: "Foreign Work & Education Matrix (Decision Table)".to_string(),
        description: Some("Skill transferability scored via spreadsheet decision table".to_string()),
        phase: "scoring".to_string(),
        category: "skill_transferability".to_string(),
        priority: 100,
        activation_group: None,
        no_loop: false,
        is_eligibility_gate: false,
        enabled: true,
        condition: Condition::Always,
        actions: vec![Action::AwardPoints {
            category: "skill_transferability".to_string(),
            formula: PointsFormula::DecisionTable(Box::new(decision_table.clone())),
            reason: "Evaluated via CSV Decision Table".to_string(),
        }],
    };

    let program = RuleProgram {
        id: "decision_table_program".to_string(),
        name: "Decision Table Express Suite".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Demonstrating multi-column Decision Table hit policies".to_string()),
        categories: std::collections::HashMap::new(),
        total_points_cap: None,
        pass_mark_threshold: None,
        rules: vec![dt_rule],
    };

    // 3. Create test applicant context
    let fact_data = json!({
        "applicant": {
            "education": {
                "highest_degree": "master"
            },
            "work_experience": {
                "foreign_years": 3
            },
            "language": {
                "first_official": {
                    "clb_reading": 9
                }
            }
        }
    });
    let context = FactContext::from_value(fact_data);

    // 4. Evaluate with Engine
    let engine = Engine::new(program);
    let report = engine.evaluate(&context)?;

    // 5. Direct Decision Table Hit Policy tests
    println!("\nCandidate Attributes:");
    println!(" - Highest Degree: Master");
    println!(" - Foreign Experience: 3 Years");
    println!(" - CLB Reading: 9");

    println!("\n--- Hit Policy Comparison ---");
    
    // First
    let mut dt_test = decision_table.clone();
    dt_test.hit_policy = HitPolicy::First;
    let score_first = engine.evaluate_decision_table(&context, &dt_test)?;
    println!("HitPolicy::First       -> {:.1} pts (First matching row)", score_first);

    // CollectSum
    dt_test.hit_policy = HitPolicy::CollectSum;
    let score_sum = engine.evaluate_decision_table(&context, &dt_test)?;
    println!("HitPolicy::CollectSum  -> {:.1} pts (Sum of all matching rows)", score_sum);

    // CollectMax
    dt_test.hit_policy = HitPolicy::CollectMax;
    let score_max = engine.evaluate_decision_table(&context, &dt_test)?;
    println!("HitPolicy::CollectMax  -> {:.1} pts (Max value of matching rows)", score_max);

    println!("\n--- Audit Report Output ---");
    report.print_audit_table();

    Ok(())
}
