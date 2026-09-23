//! Backward Chaining Goal-Driven Reasoning Demo with KSD-CO/rust-rule-engine
//!
//! Demonstrates:
//! - Goal-driven query formulation: Can a target goal be proven?
//! - Unification and recursive backward inference across dependent rules
//! - Query result verification (`result.provable`)
//! - Inspecting missing facts when a goal cannot be proven
//! - Proof trace and explanation extraction
//!
//! Run with:
//!   cargo run --example rust_rule_engine_backward_chaining

use colored::Colorize;
use rust_rule_engine::backward::BackwardEngine;
use rust_rule_engine::types::Value;
use rust_rule_engine::{Facts, KnowledgeBase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🎯 KSD-CO/rust-rule-engine: BACKWARD CHAINING DEMO 🎯 "
            .bold()
            .on_green()
            .black()
    );
    println!(" Demonstrates goal-driven reasoning, automated proof discovery, and explanation trees.\n");

    // =========================================================================
    // 1. Goal-Driven Immigration Visa Approval Query
    // =========================================================================
    println!(
        "{}",
        "1️⃣  Goal-Driven Visa Fast-Track Proof..."
            .bold()
            .underline()
    );

    let kb = KnowledgeBase::new("VisaBackwardChaining");

    let rules = r#"
    rule "FastTrackEligibility" salience 100 {
        when
            Applicant.HasApprovedSponsor == true && Applicant.MeetsThreshold == true
        then
            Applicant.FastTrackApproved = true;
    }

    rule "VerifyThreshold" salience 90 {
        when
            Applicant.Salary >= 38700.0 && Applicant.SkillLevel >= 4
        then
            Applicant.MeetsThreshold = true;
    }

    rule "VerifySponsorLicense" salience 80 {
        when
            Sponsor.Status == "Active_A_Rated"
        then
            Applicant.HasApprovedSponsor = true;
    }
    "#;

    kb.add_rules_from_grl(rules)?;

    println!("   • Loaded {} rules into KnowledgeBase.", kb.get_rules().len().to_string().cyan());

    let mut bc_engine = BackwardEngine::new(kb.clone());

    // Provide initial base facts (do NOT provide derived facts like FastTrackApproved)
    let mut facts = Facts::new();
    facts.set("Applicant.Salary", Value::Number(45000.0));
    facts.set("Applicant.SkillLevel", Value::Integer(4));
    facts.set("Sponsor.Status", Value::String("Active_A_Rated".to_string()));

    println!("   • Working Memory Base Facts:");
    println!("     - Applicant.Salary: £45,000");
    println!("     - Applicant.SkillLevel: RQF 4");
    println!("     - Sponsor.Status: Active_A_Rated");

    // Goal: Can we prove that FastTrackApproved is true?
    let goal_query = "Applicant.FastTrackApproved == true";
    println!("\n   🔍 Querying Goal: '{}'...", goal_query.yellow());

    let result = bc_engine.query(goal_query, &mut facts)?;

    if result.provable {
        println!("   {} Goal is PROVABLE!", "✅ SUCCESS:".bold().green());
        println!("   • Goals Explored: {}", result.stats.goals_explored);
        println!("   • Rules Evaluated: {}", result.stats.rules_evaluated);
        println!("   • Max Search Depth: {}", result.stats.max_depth);
        println!("   • Execution Time: {:?} ms", result.stats.duration_ms);
        println!("   • Proof Trace: {:?}", result.proof_trace);
    } else {
        println!("   {} Goal could not be proven.", "❌ FAILED:".bold().red());
        println!("   • Missing Facts: {:?}", result.missing_facts);
    }

    // =========================================================================
    // 2. Demonstrating Incomplete Facts / Missing Evidence
    // =========================================================================
    println!(
        "\n{}",
        "2️⃣  Querying Goal with Missing Base Evidence (Unprovable Goal)..."
            .bold()
            .underline()
    );

    let mut incomplete_facts = Facts::new();
    incomplete_facts.set("Applicant.Salary", Value::Number(25000.0)); // Too low
    incomplete_facts.set("Sponsor.Status", Value::String("Suspended".to_string())); // Not A-rated

    println!("   🔍 Querying Goal on Under-qualified Candidate: '{}'...", goal_query.yellow());
    let mut failed_engine = BackwardEngine::new(kb);
    let failed_result = failed_engine.query(goal_query, &mut incomplete_facts)?;

    if failed_result.provable {
        println!("   Unexpectedly proved goal!");
    } else {
        println!("   {} Goal correctly rejected as unprovable!", "🛡️ REJECTED:".bold().green());
        println!("   • Goals Explored: {}", failed_result.stats.goals_explored);
        println!("   • Rules Evaluated: {}", failed_result.stats.rules_evaluated);
        println!("   • Missing Proof Chain Identified.");
    }

    println!("\n════════════════════════════════════════════════════════════════════════════");
    println!(" 🎯 Key Takeaway: KSD-CO/rust-rule-engine backward chaining works backwards");
    println!("    from hypotheses to facts, executing goal discovery without forward iteration!");
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
