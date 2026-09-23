//! Real-time Event Streaming & CEP Demo with KSD-CO/rust-rule-engine
//!
//! Demonstrates:
//! - TimeWindow sliding/tumbling window event processing
//! - Generating and capturing stream events with timestamps
//! - Combining stream aggregation with rule engine forward-chaining
//! - Real-time compliance and fraud velocity checks
//!
//! Run with:
//!   cargo run --example rust_rule_engine_streaming

use colored::Colorize;
use rust_rule_engine::streaming::{StreamEvent, TimeWindow, WindowType};
use rust_rule_engine::types::Value;
use rust_rule_engine::{Facts, KnowledgeBase, RustRuleEngine};
use std::collections::HashMap;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🌊 KSD-CO/rust-rule-engine: STREAMING & CEP ENGINE DEMO 🌊 "
            .bold()
            .on_cyan()
            .black()
    );
    println!(" Demonstrates real-time event windows, velocity rules, and stream operators.\n");

    // =========================================================================
    // 1. Sliding Time Window: Velocity Check (e.g. rapid transactions)
    // =========================================================================
    println!(
        "{}",
        "1️⃣  Sliding Time Window Velocity Aggregation..."
            .bold()
            .underline()
    );

    let start_ts = 1_700_000_000_000u64; // base millisecond timestamp
    let window_duration = Duration::from_secs(60); // 60-second sliding window
    let mut window = TimeWindow::new(WindowType::Sliding, window_duration, start_ts, 10_000);

    // Record rapid transaction events
    for i in 1..=5 {
        let mut data = HashMap::new();
        data.insert("amount".to_string(), Value::Number(500.0 * i as f64));
        data.insert("user_id".to_string(), Value::String("USER-99".to_string()));
        let event_ts = start_ts + (i * 5_000); // every 5 seconds
        let event = StreamEvent::with_timestamp("card_swipe", data, "pos_terminal", event_ts);
        window.record(event);
    }

    println!("   • Window Duration: 60s sliding");
    println!("   • Total Events in Window: {}", window.count().to_string().cyan());
    println!("   • Window Interval: [{}..{}] ms", window.start_time, window.end_time);

    // =========================================================================
    // 2. Stream Operators with Rule Engine Decision
    // =========================================================================
    println!(
        "\n{}",
        "2️⃣  Evaluating Velocity Event Stream with Rule Engine..."
            .bold()
            .underline()
    );

    let kb = KnowledgeBase::new("VelocityRules");
    let rules = r#"
    rule "RapidTransactionsFraudAlert" salience 100 no-loop {
        when
            Velocity.Count >= 5 && Velocity.TotalAmount > 5000.0
        then
            Alert.Triggered = true;
            Alert.Reason = "Rapid high-value transaction burst detected";
            Log("FRAUD SUSPICION: Burst of 5+ transactions exceeding £5,000 in 60s window!");
    }
    "#;
    kb.add_rules_from_grl(rules)?;

    let mut engine = RustRuleEngine::new(kb);

    let mut velocity_data = HashMap::new();
    velocity_data.insert("Count".to_string(), Value::Integer(window.count() as i64));
    velocity_data.insert("TotalAmount".to_string(), Value::Number(7500.0));

    let mut alert_data = HashMap::new();
    alert_data.insert("Triggered".to_string(), Value::Boolean(false));
    alert_data.insert("Reason".to_string(), Value::String("None".to_string()));

    let facts = Facts::new();
    facts.add_value("Velocity", Value::Object(velocity_data))?;
    facts.add_value("Alert", Value::Object(alert_data))?;

    let result = engine.execute(&facts)?;
    println!(
        "   • Rule Evaluation: {} rules fired. Alert State: {:?}",
        result.rules_fired,
        facts.get("Alert")
    );

    println!("\n════════════════════════════════════════════════════════════════════════════");
    println!(" ⚡ Performance: Sliding windows and CEP stream operators natively integrate");
    println!("    with KSD-CO/rust-rule-engine working memory for real-time reactive rules!");
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
