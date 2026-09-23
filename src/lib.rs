pub mod core;
pub mod immigration;

pub use core::{
    Action, AuditReport, CategoryConfig, CategoryScoreBreakdown, ComparisonOperator, Condition,
    DecisionTable, DecisionTableInput, DecisionTableOutput, DecisionTableRow, Engine, EngineError,
    FactContext, FiredRuleRecord, HitPolicy, PointsFormula, Result, Rule, RuleProgram,
};

pub use immigration::{
    AdditionalFactors, ApplicantFactWrapper, ApplicantProfile, EducationCredential, EmployerSponsor,
    JobOffer, LanguageAbilityScore, LanguageProficiency, SpouseProfile, WorkExperience,
};

