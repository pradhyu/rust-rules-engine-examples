//! Quickstart Example for KSD-CO/rust-rule-engine
//!
//! Demonstrates:
//! - Defining rules in GRL (Grule Rule Language)
//! - Loading rules via `RuleEngineBuilder` and `KnowledgeBase`
//! - Setting facts with nested fields and types
//! - Forward-chaining execution and reading results
//!
//! Run with:
//!   cargo run --example rust_rule_engine_quickstart

use colored::Colorize;
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RuleEngineBuilder, RustRuleEngine, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🚀 KSD-CO/rust-rule-engine: QUICKSTART DEMO 🚀 "
            .bold()
            .on_blue()
            .white()
    );
    println!(" Demonstrates GRL rule parsing, facts management, and forward-chaining inference.\n");

    // =========================================================================
    // 1. Fluent Builder Pattern (RuleEngineBuilder)
    // =========================================================================
    println!(
        "{}",
        "1️⃣  Executing Rules via Fluent RuleEngineBuilder..."
            .bold()
            .underline()
    );

    let grl_rules = r#"
        rule "VIPDiscount" salience 10 no-loop {
            when
                Customer.TotalSpent >= 10000 && Customer.IsLoyal == true
            then
                Customer.Discount = 0.20;
                Customer.Tier = "VIP Platinum";
                Log("Customer promoted to VIP Platinum with 20% discount!");
        }

        rule "StandardDiscount" salience 5 no-loop {
            when
                Customer.TotalSpent >= 2500 && Customer.TotalSpent < 10000
            then
                Customer.Discount = 0.10;
                Customer.Tier = "Gold";
                Log("Customer awarded Gold Tier with 10% discount.");
        }
    "#;

    let mut engine = RuleEngineBuilder::new()
        .with_inline_grl(grl_rules)?
        .build();

    let facts = Facts::new();
    let customer_json = serde_json::json!({
        "TotalSpent": 15000,
        "IsLoyal": true,
        "Discount": 0.0,
        "Tier": "Standard"
    });
    facts.add_value("Customer", Value::from(customer_json))?;

    println!("   • Initial Customer: {:?}", facts.get("Customer"));
    let result = engine.execute(&facts)?;
    println!(
        "   • Execution: {} rules evaluated, {} fired across {} cycle(s) in {:?}",
        result.rules_evaluated, result.rules_fired, result.cycle_count, result.execution_time
    );
    println!("   • Updated Customer: {:?}\n", facts.get("Customer"));

    // =========================================================================
    // 2. Direct KnowledgeBase & Custom Action Handlers
    // =========================================================================
    println!(
        "{}",
        "2️⃣  Custom Action Handlers & Working Memory Updates..."
            .bold()
            .underline()
    );

    let fraud_grl = r#"
        rule "HighRiskTransaction" salience 100 no-loop {
            when
                Transaction.Amount > 5000 && Transaction.Country != "DOMESTIC"
            then
                Alert.Flagged = true;
                Log("ALERT: High risk cross-border transaction detected!");
        }
    "#;

    let rules = GRLParser::parse_rules(fraud_grl)?;
    let kb = KnowledgeBase::new("FraudProtection");
    for rule in rules {
        kb.add_rule(rule)?;
    }

    let mut fraud_engine = RustRuleEngine::new(kb);
    let txn_facts = Facts::new();
    txn_facts.add_value(
        "Transaction",
        Value::from(serde_json::json!({
            "Amount": 12500,
            "Country": "INTERNATIONAL",
            "Currency": "USD"
        })),
    )?;
    txn_facts.add_value(
        "Alert",
        Value::from(serde_json::json!({
            "Flagged": false
        })),
    )?;

    println!("   • Initial Alert State: {:?}", txn_facts.get("Alert"));
    let txn_res = fraud_engine.execute(&txn_facts)?;
    println!(
        "   • Execution: {} rules fired. Updated Alert: {:?}\n",
        txn_res.rules_fired,
        txn_facts.get("Alert")
    );

    println!("════════════════════════════════════════════════════════════════════════════");
    println!(" ✅ Success: KSD-CO/rust-rule-engine evaluated facts seamlessly with GRL syntax!");
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
