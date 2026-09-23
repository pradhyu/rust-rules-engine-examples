#![allow(clippy::result_large_err)]

pub mod core;
pub mod immigration;
pub mod repl;
pub mod server;

pub use core::{
    Action, AuditReport, CategoryConfig, CategoryScoreBreakdown, ComparisonOperator, Condition,
    Engine, EngineError, FactContext, FiredRuleRecord, PointsFormula, Result, Rule, RuleProgram,
};

pub use immigration::{
    AdditionalFactors, ApplicantFactWrapper, ApplicantProfile, EducationCredential,
    EmployerSponsor, JobOffer, LanguageAbilityScore, LanguageProficiency, SpouseProfile,
    WorkExperience,
};

pub use repl::run_interactive_repl;

pub use server::{
    AppState, BatchApiRequest, BatchApiResponse, BatchRequestDto, BatchResponseDto,
    EvaluateApiRequest, EvaluateApiResponse, EvaluateRequestDto, EvaluateResponseDto,
    HealthCheckResponseDto, PathwayOptionDto, RankedCandidateDto, RulesGrpcService,
    RulesServiceServer, RulesetInspectionDto, SimulateApiRequest, SimulateApiResponse,
    SimulateRequestDto, SimulateResponseDto, create_rest_router, dto, proto,
};
