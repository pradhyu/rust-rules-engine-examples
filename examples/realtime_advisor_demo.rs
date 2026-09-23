use colored::Colorize;
use rust_rules_engine::{
    AdditionalFactors, ApplicantProfile, EducationCredential,
    LanguageAbilityScore, LanguageProficiency, WorkExperience,
    evaluate_facts, json_to_facts, load_knowledge_base_from_path,
};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🍁 REAL-TIME IMMIGRATION ADVISOR & RECOMMENDATION ENGINE (via KSD-CO/rust-rule-engine) 🍁 "
            .bold()
            .on_blue()
            .white()
    );
    println!(" This example demonstrates evaluating a single person's data in microseconds");
    println!(" and running 'What-If' simulations to output actionable recommendations.\n");

    // 1. Load the KnowledgeBase once (cached in memory)
    let load_start = Instant::now();
    let (kb, pass_mark) = load_knowledge_base_from_path("rules/canada_crs_express_entry.grl")?;
    println!("Knowledge Base initialized in: {:?}\n", load_start.elapsed());

    // 2. Baseline Applicant Profile (Candidate with moderate language score)
    let baseline_profile = ApplicantProfile {
        id: "CAN-APP-2026-8941".to_string(),
        first_name: "Sophia".to_string(),
        last_name: "Chen".to_string(),
        age: 29,
        marital_status: "single".to_string(),
        education: EducationCredential {
            highest_degree: "master".to_string(),
            field_of_study: Some("Data Engineering".to_string()),
            is_stem: true,
            is_stem_research: false,
            is_domestic_study: false,
            domestic_study_years: None,
        },
        language: LanguageProficiency {
            first_official: Some(LanguageAbilityScore {
                clb_reading: 7,
                clb_writing: 7,
                clb_listening: 7,
                clb_speaking: 7,
                test_type: Some("IELTS".to_string()),
                composite_clb: Some(7.0),
            }),
            second_official: None,
            cefr_level: Some("B2".to_string()),
            english_tier: None,
            test_date: Some("2025-11-15".to_string()),
            test_age_days: Some(120),
        },
        work_experience: WorkExperience {
            domestic_years: 1,
            foreign_years: 3,
            primary_noc_code: Some("21231".to_string()),
            skill_level: Some("TEER 1".to_string()),
            has_trade_certification: false,
        },
        job_offer: None,
        spouse: None,
        additional_factors: Some(AdditionalFactors {
            provincial_nomination: false,
            has_sibling_citizen_or_pr: false,
            french_speaker_bonus_eligible: false,
        }),
    };

    // 3. Real-time Baseline Evaluation
    let eval_start = Instant::now();
    let baseline_facts =
        json_to_facts(&serde_json::json!({ "applicant": &baseline_profile }));
    let baseline_report = evaluate_facts(&kb, &baseline_facts, pass_mark)?;
    let eval_duration = eval_start.elapsed();

    let baseline_score = baseline_report.total_score;
    let recent_cutoff = 485.0; // Simulated Express Entry draw cutoff

    println!("══════════════════════════════════════════════════════════════════════");
    println!(
        " 👤 CANDIDATE PROFILE: {} {}",
        baseline_profile.first_name.bold(),
        baseline_profile.last_name.bold()
    );
    println!(
        "    ID: {} | Age: {} | Education: Master's Degree",
        baseline_profile.id.cyan(),
        baseline_profile.age
    );
    println!("    Experience: 1 yr Canadian + 3 yrs Foreign | Language: CLB 7");
    println!("──────────────────────────────────────────────────────────────────────");
    println!(" ⚡ Real-Time Evaluation Latency: {:?}", eval_duration);
    println!(
        " 📊 Current CRS Score: {} / 1200 points",
        format!("{:.1}", baseline_score).yellow().bold()
    );
    println!(" 🎯 Target Draw Cutoff: {:.1} points", recent_cutoff);

    let gap = recent_cutoff - baseline_score;
    if gap > 0.0 {
        println!(
            " ⚠️ Status: {} (Need {:.1} more points to qualify for General Draw)",
            "BELOW CUTOFF".red().bold(),
            gap
        );
    } else {
        println!(
            " ✅ Status: {}",
            "COMPETITIVE (Above Cutoff)".green().bold()
        );
    }
    println!("══════════════════════════════════════════════════════════════════════\n");

    // 4. Real-time Pathway Simulations ("What-If" Analysis)
    println!(
        "{}",
        "💡 RUNNING REAL-TIME 'WHAT-IF' PATHWAY ADVISORY SIMULATIONS..."
            .bold()
            .underline()
    );

    struct Recommendation {
        title: String,
        action_item: String,
        projected_score: f64,
        point_gain: f64,
        reaches_cutoff: bool,
    }

    let mut recommendations = Vec::new();

    // Simulation A: Retake IELTS to achieve CLB 9+ in all 4 abilities
    {
        let mut sim_profile = baseline_profile.clone();
        if let Some(ref mut lang) = sim_profile.language.first_official {
            lang.clb_reading = 9;
            lang.clb_writing = 9;
            lang.clb_listening = 9;
            lang.clb_speaking = 9;
            lang.composite_clb = Some(9.0);
        }
        let sim_facts = json_to_facts(&serde_json::json!({ "applicant": sim_profile }));
        let sim_report = evaluate_facts(&kb, &sim_facts, pass_mark)?;
        let gain = sim_report.total_score - baseline_score;
        recommendations.push(Recommendation {
            title: "Retake Language Exam to reach CLB 9+ (Reading 8.0, L/W/S 7.0+)".to_string(),
            action_item: "Unlocks maximum Skill Transferability multipliers for Master's + Foreign Work experience.".to_string(),
            projected_score: sim_report.total_score,
            point_gain: gain,
            reaches_cutoff: sim_report.total_score >= recent_cutoff,
        });
    }

    // Simulation B: Gain 1 additional year of Canadian Work Experience (Total 2 years)
    {
        let mut sim_profile = baseline_profile.clone();
        sim_profile.work_experience.domestic_years = 2;
        let sim_facts = json_to_facts(&serde_json::json!({ "applicant": sim_profile }));
        let sim_report = evaluate_facts(&kb, &sim_facts, pass_mark)?;
        let gain = sim_report.total_score - baseline_score;
        recommendations.push(Recommendation {
            title: "Complete 2nd Year of Canadian Domestic Work Experience".to_string(),
            action_item: "Increases Core Human Capital domestic work points from 40 to 53."
                .to_string(),
            projected_score: sim_report.total_score,
            point_gain: gain,
            reaches_cutoff: sim_report.total_score >= recent_cutoff,
        });
    }

    // Simulation C: Sibling in Canada (Bonus Factor)
    {
        let mut sim_profile = baseline_profile.clone();
        sim_profile.additional_factors = Some(AdditionalFactors {
            provincial_nomination: false,
            has_sibling_citizen_or_pr: true,
            french_speaker_bonus_eligible: false,
        });
        let sim_facts = json_to_facts(&serde_json::json!({ "applicant": sim_profile }));
        let sim_report = evaluate_facts(&kb, &sim_facts, pass_mark)?;
        let gain = sim_report.total_score - baseline_score;
        recommendations.push(Recommendation {
            title: "Claim Canadian Citizen / Permanent Resident Sibling Bonus".to_string(),
            action_item: "Provides an instant 15-point direct regulatory bonus.".to_string(),
            projected_score: sim_report.total_score,
            point_gain: gain,
            reaches_cutoff: sim_report.total_score >= recent_cutoff,
        });
    }

    // Simulation D: Apply for Provincial Nomination (PNP Tech Draw)
    {
        let mut sim_profile = baseline_profile.clone();
        sim_profile.additional_factors = Some(AdditionalFactors {
            provincial_nomination: true,
            has_sibling_citizen_or_pr: false,
            french_speaker_bonus_eligible: false,
        });
        let sim_facts = json_to_facts(&serde_json::json!({ "applicant": sim_profile }));
        let sim_report = evaluate_facts(&kb, &sim_facts, pass_mark)?;
        let gain = sim_report.total_score - baseline_score;
        recommendations.push(Recommendation {
            title: "Secure an Enhanced Provincial Nomination (e.g. Ontario Tech Draw)".to_string(),
            action_item: "Awarded +600 points, guaranteeing an Invitation to Apply (ITA) in the very next draw round.".to_string(),
            projected_score: sim_report.total_score,
            point_gain: gain,
            reaches_cutoff: sim_report.total_score >= recent_cutoff,
        });
    }

    // Sort recommendations by point gain descending
    recommendations.sort_by(|a, b| {
        b.point_gain
            .partial_cmp(&a.point_gain)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Print Recommendation Cards
    for (i, rec) in recommendations.iter().enumerate() {
        let rank_str = format!("{}.", i + 1).bold();
        let gain_str = format!("+{:.1} pts", rec.point_gain).green().bold();
        let projected_str = format!("{:.1} pts", rec.projected_score).cyan().bold();

        let qualifier_badge = if rec.reaches_cutoff {
            " 🌟 QUALIFIES FOR INVITATION (ITA) 🌟 ".bold().on_green().white()
        } else {
            " 📈 PARTIAL IMPROVEMENT ".bold().on_yellow().black()
        };

        println!("╭────────────────────────────────────────────────────────────────────╮");
        println!(
            "│ {}  {:<50} │",
            rank_str,
            rec.title.bold()
        );
        println!("│                                                                    │");
        println!(
            "│    Points Gain:      {:<45} │",
            gain_str
        );
        println!(
            "│    Projected Total:  {:<45} │",
            projected_str
        );
        println!(
            "│    Status:           {:<45} │",
            qualifier_badge
        );
        println!("│                                                                    │");
        println!(
            "│    Details: {:<54} │",
            rec.action_item.dimmed()
        );
        println!("╰────────────────────────────────────────────────────────────────────╯");
    }

    println!("\nSummary: Evaluated baseline candidate and simulated 4 strategic immigration pathways.");
    Ok(())
}
