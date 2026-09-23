use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAbilityScore {
    pub clb_reading: i64,
    pub clb_writing: i64,
    pub clb_listening: i64,
    pub clb_speaking: i64,
    #[serde(default)]
    pub test_type: Option<String>,
    #[serde(default)]
    pub composite_clb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageProficiency {
    pub first_official: Option<LanguageAbilityScore>,
    #[serde(default)]
    pub second_official: Option<LanguageAbilityScore>,
    #[serde(default)]
    pub cefr_level: Option<String>,
    #[serde(default)]
    pub english_tier: Option<String>,
    #[serde(default)]
    pub test_date: Option<String>,
    #[serde(default)]
    pub test_age_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationCredential {
    pub highest_degree: String,
    #[serde(default)]
    pub field_of_study: Option<String>,
    #[serde(default)]
    pub is_stem: bool,
    #[serde(default)]
    pub is_stem_research: bool,
    #[serde(default)]
    pub is_domestic_study: bool,
    #[serde(default)]
    pub domestic_study_years: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkExperience {
    pub domestic_years: i64,
    pub foreign_years: i64,
    #[serde(default)]
    pub primary_noc_code: Option<String>,
    #[serde(default)]
    pub skill_level: Option<String>,
    #[serde(default)]
    pub has_trade_certification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployerSponsor {
    #[serde(default)]
    pub is_licensed: bool,
    #[serde(default)]
    pub license_rating: Option<String>,
    #[serde(default)]
    pub license_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOffer {
    pub has_offer: bool,
    #[serde(default)]
    pub is_lmia_approved_or_exempt: bool,
    #[serde(default)]
    pub noc_teer: Option<String>,
    #[serde(default)]
    pub rqf_skill_level: Option<i64>,
    #[serde(default)]
    pub annual_salary: Option<f64>,
    #[serde(default)]
    pub meets_occupation_going_rate: bool,
    #[serde(default)]
    pub is_on_immigration_salary_list: bool,
    #[serde(default)]
    pub sponsor: Option<EmployerSponsor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalFactors {
    #[serde(default)]
    pub provincial_nomination: bool,
    #[serde(default)]
    pub has_sibling_citizen_or_pr: bool,
    #[serde(default)]
    pub french_speaker_bonus_eligible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpouseProfile {
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub education: Option<EducationCredential>,
    #[serde(default)]
    pub language: Option<LanguageProficiency>,
    #[serde(default)]
    pub has_positive_skills_assessment: bool,
    #[serde(default)]
    pub english_competent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicantProfile {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub age: i64,
    pub marital_status: String,
    pub education: EducationCredential,
    pub language: LanguageProficiency,
    pub work_experience: WorkExperience,
    #[serde(default)]
    pub job_offer: Option<JobOffer>,
    #[serde(default)]
    pub additional_factors: Option<AdditionalFactors>,
    #[serde(default)]
    pub spouse: Option<SpouseProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicantFactWrapper {
    pub applicant: ApplicantProfile,
}
