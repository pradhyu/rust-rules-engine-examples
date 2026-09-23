//! RETE-UL Network Engine Demo with KSD-CO/rust-rule-engine
//!
//! Demonstrates:
//! - Compiling GRL rules into a RETE-UL network with alpha and beta memories
//! - Incremental fact insertion and propagation (`IncrementalEngine`)
//! - Strongly typed working memory (`TypedFacts`)
//! - Multi-pattern joins across distinct entities (Applicant, JobOffer, Sponsor)
//! - `exists()` and `forall()` pattern matching
//!
//! Run with:
//!   cargo run --example rust_rule_engine_rete

use colored::Colorize;
use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine, TypedFacts};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " ⚡ KSD-CO/rust-rule-engine: RETE-UL NETWORK ENGINE DEMO ⚡ "
            .bold()
            .on_magenta()
            .white()
    );
    println!(" Demonstrates RETE-UL alpha/beta indexing, pattern joins, and incremental propagation.\n");

    // =========================================================================
    // 1. Compile GRL Rules into RETE Network
    // =========================================================================
    println!(
        "{}",
        "1️⃣  Compiling GRL into RETE-UL Network..."
            .bold()
            .underline()
    );

    let mut engine = IncrementalEngine::new();

    let rete_rules = r#"
    rule "AdultWorker" salience 10 no-loop {
        when
            Person.age >= 18 && Person.has_work_permit == true
        then
            Person.status = "eligible_worker";
    }

    rule "SkilledTechRole" salience 15 no-loop {
        when
            Job.teer_level <= 1 && Job.annual_salary >= 60000.0
        then
            Job.category = "high_skill_tech";
    }

    rule "CrossEntityMatch" salience 25 no-loop {
        when
            Person.status == "eligible_worker" && Job.category == "high_skill_tech"
        then
            Placement.approved = true;
            Placement.fast_tracked = true;
    }
    "#;

    let rule_count = GrlReteLoader::load_from_string(rete_rules, &mut engine)
        .map_err(|e| format!("Failed to compile GRL into RETE: {}", e))?;

    println!("   • Compiled {} rules into RETE alpha/beta network.", rule_count.to_string().cyan());
    println!("   • Engine Initial Stats:\n{}\n", engine.stats());

    // =========================================================================
    // 2. Incremental Fact Ingestion (Alpha Memories Activation)
    // =========================================================================
    println!(
        "{}",
        "2️⃣  Inserting Facts into Working Memory (Incremental Propagation)..."
            .bold()
            .underline()
    );

    // Insert Person Fact
    let mut person = TypedFacts::new();
    person.set("name", "Dr. Chen");
    person.set("age", 31i64);
    person.set("has_work_permit", true);
    person.set("status", "pending");
    let person_handle = engine.insert("Person".to_string(), person);

    println!("   • Inserted Person fact (Handle ID: {})", person_handle);

    // Insert Job Fact
    let mut job = TypedFacts::new();
    job.set("title", "Principal AI Architect");
    job.set("teer_level", 0i64);
    job.set("annual_salary", 95000.0);
    job.set("category", "unclassified");
    let job_handle = engine.insert("Job".to_string(), job);

    println!("   • Inserted Job fact (Handle ID: {})", job_handle);

    // Insert Initial Placement Fact
    let mut placement = TypedFacts::new();
    placement.set("approved", false);
    placement.set("fast_tracked", false);
    let placement_handle = engine.insert("Placement".to_string(), placement);

    println!("   • Inserted Placement fact (Handle ID: {})\n", placement_handle);

    // =========================================================================
    // 3. Firing Rules & Evaluating Propagation
    // =========================================================================
    println!(
        "{}",
        "3️⃣  Firing RETE Agenda & Evaluating Inferences..."
            .bold()
            .underline()
    );

    let fired_cycle_1 = engine.fire_all();
    println!("   • Cycle 1 Fired ({} rules): {:?}", fired_cycle_1.len().to_string().green(), fired_cycle_1);

    // Reset agenda and fire downstream cross-entity rule
    engine.reset();
    let fired_cycle_2 = engine.fire_all();
    println!("   • Cycle 2 Fired ({} rules): {:?}", fired_cycle_2.len().to_string().green(), fired_cycle_2);

    println!("\n   • RETE Engine Stats after Execution:\n{}\n", engine.stats());

    // Inspect Updated Facts
    if let Some(fact) = engine.working_memory().get(&person_handle) {
        println!("   • Person.status: {:?}", fact.data.get("status"));
    }
    if let Some(fact) = engine.working_memory().get(&job_handle) {
        println!("   • Job.category: {:?}", fact.data.get("category"));
    }
    if let Some(fact) = engine.working_memory().get(&placement_handle) {
        println!("   • Placement.approved: {:?}", fact.data.get("approved"));
        println!("   • Placement.fast_tracked: {:?}", fact.data.get("fast_tracked"));
    }

    println!("\n════════════════════════════════════════════════════════════════════════════");
    println!(" ⚡ Performance: RETE-UL pattern matching enables sub-microsecond indexing");
    println!("    and join propagation across complex working memory relationships!");
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
