#![allow(clippy::result_large_err)]

pub mod evaluator;
pub mod immigration;
pub mod repl;
pub mod server;

/// Re-export upstream KSD-CO/rust-rule-engine
pub use rust_rule_engine;
pub use rust_rule_engine::{
    EngineConfig, Facts, GRLParser, GruleExecutionResult, KnowledgeBase, Rule, RuleEngineBuilder,
    RustRuleEngine, Value,
};

pub use evaluator::{
    AuditReport, CategoryConfig, CategoryScoreBreakdown, FiredRuleRecord, evaluate_facts,
    json_to_facts, load_knowledge_base_from_path,
};

pub use immigration::{
    AdditionalFactors, ApplicantFactWrapper, ApplicantProfile, EducationCredential,
    EmployerSponsor, JobOffer, LanguageAbilityScore, LanguageProficiency, SpouseProfile,
    WorkExperience,
};

pub use repl::{LoadedRuleset, run_interactive_repl};

pub use server::{
    AppState, BatchApiRequest, BatchApiResponse, BatchRequestDto, BatchResponseDto,
    EvaluateApiRequest, EvaluateApiResponse, EvaluateRequestDto, EvaluateResponseDto,
    HealthCheckResponseDto, PathwayOptionDto, RankedCandidateDto, RulesGrpcService,
    RulesServiceServer, RulesetInspectionDto, SimulateApiRequest, SimulateApiResponse,
    SimulateRequestDto, SimulateResponseDto, create_rest_router, dto, proto,
};
