use rust_rules_engine::{
    ApplicantProfile, EducationCredential, Engine, FactContext, LanguageAbilityScore,
    LanguageProficiency, RuleProgram, WorkExperience,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍁 Canada Express Entry CRS Programmatic API Demo 🍁\n");

    // 1. Load declarative RuleProgram from YAML
    let program = RuleProgram::from_yaml_file("rules/canada_crs_express_entry.yaml")?;
    let engine = Engine::new(program);

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

    // 3. Ingest profile into FactContext (under "applicant" root)
    let fact_wrapper = serde_json::json!({ "applicant": profile });
    let context = FactContext::from_value(fact_wrapper);

    // 4. Evaluate and generate complete audit report
    let report = engine.evaluate(&context)?;

    // 5. Inspect structured result
    println!("Candidate ID: {:?}", report.applicant_id);
    println!("Eligibility: {}", report.is_eligible());
    println!("Calculated CRS Score: {:.1} / 1200", report.total_score);

    // 6. Print console audit breakdown
    report.print_audit_table();

    Ok(())
}
