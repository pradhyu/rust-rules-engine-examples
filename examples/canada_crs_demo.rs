use rust_rules_engine::{
    ApplicantProfile, EducationCredential, LanguageAbilityScore, LanguageProficiency,
    WorkExperience, evaluate_facts, json_to_facts, load_knowledge_base_from_path,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍁 Canada Express Entry CRS Programmatic API Demo (via KSD-CO/rust-rule-engine) 🍁\n");

    // 1. Load KnowledgeBase from GRL
    let (kb, pass_mark) = load_knowledge_base_from_path("rules/canada_crs_express_entry.grl")?;

    // 2. Create strongly-typed ApplicantProfile in Rust
    let profile = ApplicantProfile {
        id: "CAN-DEV-001".to_string(),
        first_name: "Liam".to_string(),
        last_name: "Tremblay".to_string(),
        age: 28,
        marital_status: "single".to_string(),
        education: EducationCredential {
            highest_degree: "master".to_string(),
            field_of_study: Some("Computer Science".to_string()),
            is_stem: true,
            is_stem_research: false,
            is_domestic_study: false,
            domestic_study_years: None,
        },
        language: LanguageProficiency {
            first_official: Some(LanguageAbilityScore {
                clb_reading: 9,
                clb_writing: 9,
                clb_listening: 9,
                clb_speaking: 9,
                test_type: Some("ielts".to_string()),
                composite_clb: Some(9.0),
            }),
            second_official: None,
            cefr_level: None,
            english_tier: None,
            test_date: None,
            test_age_days: None,
        },
        work_experience: WorkExperience {
            domestic_years: 2,
            foreign_years: 3,
            primary_noc_code: Some("21232".to_string()),
            skill_level: Some("teer_1".to_string()),
            has_trade_certification: false,
        },
        job_offer: None,
        additional_factors: None,
        spouse: None,
    };

    // 3. Ingest profile into Facts (under "applicant" root)
    let fact_wrapper = serde_json::json!({ "applicant": profile });
    let facts = json_to_facts(&fact_wrapper);

    // 4. Evaluate and generate complete audit report
    let report = evaluate_facts(&kb, &facts, pass_mark)?;
    report.print_audit_table();

    Ok(())
}
