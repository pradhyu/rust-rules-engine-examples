use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use rust_rule_engine::KnowledgeBase;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::evaluator::{
    AuditReport, evaluate_facts, json_to_facts, load_knowledge_base_from_path,
};
use crate::server::dto::{
    BatchRequestDto, BatchResponseDto, EvaluateRequestDto, EvaluateResponseDto,
    HealthCheckResponseDto, PathwayOptionDto, RankedCandidateDto, RulesetInspectionDto,
    SimulateRequestDto, SimulateResponseDto,
};

// Backwards compatibility aliases
pub type EvaluateApiRequest = EvaluateRequestDto;
pub type EvaluateApiResponse = EvaluateResponseDto;
pub type BatchApiRequest = BatchRequestDto;
pub type BatchApiResponse = BatchResponseDto;
pub type SimulateApiRequest = SimulateRequestDto;
pub type SimulateApiResponse = SimulateResponseDto;

/// Shared Application State for REST & gRPC Services
#[derive(Clone)]
pub struct AppState {
    pub kb: Arc<RwLock<KnowledgeBase>>,
    pub rules_path: Arc<RwLock<String>>,
    pub pass_mark: Arc<RwLock<Option<f64>>>,
}

impl AppState {
    pub fn new(kb: KnowledgeBase, rules_path: String, pass_mark: Option<f64>) -> Self {
        Self {
            kb: Arc::new(RwLock::new(kb)),
            rules_path: Arc::new(RwLock::new(rules_path)),
            pass_mark: Arc::new(RwLock::new(pass_mark)),
        }
    }

    pub fn from_path(path: &str) -> Result<Self, crate::evaluator::EvaluatorError> {
        let (kb, pass_mark) = load_knowledge_base_from_path(path)?;
        Ok(Self::new(kb, path.to_string(), pass_mark))
    }
}

/// Create the Axum router for the REST API
pub fn create_rest_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/inspect", get(inspect_ruleset))
        .route("/api/v1/rules/reload", post(reload_ruleset))
        .route("/api/v1/evaluate", post(evaluate_applicant))
        .route("/api/v1/batch", post(batch_evaluate))
        .route("/api/v1/simulate", post(simulate_whatif))
        .with_state(state)
}

/// Health check handler
async fn health_check() -> impl IntoResponse {
    Json(HealthCheckResponseDto {
        status: "UP".to_string(),
        service: "rust-rule-engine-realtime".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Dynamic hot-reload handler to switch or update active rules at runtime
async fn reload_ruleset(
    State(state): State<AppState>,
    Json(payload): Json<crate::server::dto::ReloadRulesRequestDto>,
) -> Result<Json<crate::server::dto::ReloadRulesResponseDto>, (StatusCode, String)> {
    let (new_kb, pass_mark) = load_knowledge_base_from_path(&payload.rules_path).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed loading rules from '{}': {}", payload.rules_path, e),
        )
    })?;

    let ruleset_id = new_kb.name().to_string();
    let ruleset_name = new_kb.name().to_string();
    let version = format!("v{}", new_kb.version());
    let rule_count = new_kb.rule_count();

    {
        let mut kb_guard = state.kb.write().await;
        *kb_guard = new_kb;
    }
    {
        let mut path_guard = state.rules_path.write().await;
        *path_guard = payload.rules_path.clone();
    }
    {
        let mut pass_guard = state.pass_mark.write().await;
        *pass_guard = pass_mark;
    }

    Ok(Json(crate::server::dto::ReloadRulesResponseDto {
        success: true,
        message: format!(
            "Successfully loaded ruleset '{}' from '{}'",
            ruleset_name, payload.rules_path
        ),
        ruleset_id,
        ruleset_name,
        version,
        rule_count,
    }))
}

/// Inspect current active ruleset
async fn inspect_ruleset(State(state): State<AppState>) -> impl IntoResponse {
    let kb = state.kb.read().await;
    let pass_mark = *state.pass_mark.read().await;
    Json(RulesetInspectionDto {
        id: kb.name().to_string(),
        name: kb.name().to_string(),
        version: format!("v{}", kb.version()),
        description: Some("Knowledge Base powered by KSD-CO/rust-rule-engine".to_string()),
        pass_mark_threshold: pass_mark,
        total_points_cap: None,
        categories: HashMap::new(),
        rule_count: kb.rule_count(),
    })
}

fn normalize_fact_value(val: serde_json::Value) -> serde_json::Value {
    if val.get("applicant").is_some() {
        val
    } else {
        serde_json::json!({ "applicant": val })
    }
}

/// Real-time evaluation endpoint
async fn evaluate_applicant(
    State(state): State<AppState>,
    Json(payload): Json<EvaluateRequestDto>,
) -> Result<Json<EvaluateResponseDto>, (StatusCode, String)> {
    let start = Instant::now();
    let norm_val = normalize_fact_value(payload.applicant);
    let facts = json_to_facts(&norm_val);

    let (kb, pass_mark) = if let Some(path) = &payload.rules_path {
        load_knowledge_base_from_path(path)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    } else {
        let kb_guard = state.kb.read().await;
        let mark_guard = *state.pass_mark.read().await;
        (kb_guard.clone(), mark_guard)
    };

    let report: AuditReport = evaluate_facts(&kb, &facts, pass_mark)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let latency = start.elapsed().as_secs_f64() * 1_000_000.0;
    let eligible = report.is_eligible();

    Ok(Json(EvaluateResponseDto {
        applicant_id: report.applicant_id,
        eligible,
        total_score: report.total_score,
        pass_mark_threshold: report.pass_mark_threshold,
        ineligibility_reasons: report.ineligibility_reasons,
        fired_rules: report.fired_rules,
        category_scores: report.category_scores,
        tags: report.tags,
        latency_micros: latency,
    }))
}

/// Batch evaluation endpoint
async fn batch_evaluate(
    State(state): State<AppState>,
    Json(payload): Json<BatchRequestDto>,
) -> Result<Json<BatchResponseDto>, (StatusCode, String)> {
    let start = Instant::now();

    let (kb, pass_mark) = if let Some(path) = &payload.rules_path {
        load_knowledge_base_from_path(path)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    } else {
        let kb_guard = state.kb.read().await;
        let mark_guard = *state.pass_mark.read().await;
        (kb_guard.clone(), mark_guard)
    };

    let mut candidate_results = Vec::new();

    for (idx, app_val) in payload.applicants.into_iter().enumerate() {
        let norm_val = normalize_fact_value(app_val);
        let facts = json_to_facts(&norm_val);
        if let Ok(report) = evaluate_facts(&kb, &facts, pass_mark) {
            let candidate_id = report
                .applicant_id
                .clone()
                .unwrap_or_else(|| format!("APP-{:04}", idx + 1));
            candidate_results.push((candidate_id, report));
        }
    }

    // Sort descending by total score
    candidate_results.sort_by(|a, b| {
        b.1.total_score
            .partial_cmp(&a.1.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let candidates: Vec<RankedCandidateDto> = candidate_results
        .into_iter()
        .enumerate()
        .map(|(rank_idx, (id, report))| {
            let draw_result = if !report.is_eligible() {
                "Ineligible".to_string()
            } else if let Some(cut) = payload.cutoff {
                if report.total_score >= cut {
                    "SELECTED (ITA)".to_string()
                } else {
                    "Below Cutoff".to_string()
                }
            } else {
                "Ranked".to_string()
            };

            RankedCandidateDto {
                rank: rank_idx + 1,
                candidate_id: id,
                total_score: report.total_score,
                eligible: report.is_eligible(),
                draw_result,
            }
        })
        .collect();

    let latency = start.elapsed().as_secs_f64() * 1_000_000.0;

    Ok(Json(BatchResponseDto {
        ruleset_name: kb.name().to_string(),
        total_evaluated: candidates.len(),
        cutoff: payload.cutoff,
        candidates,
        latency_micros: latency,
    }))
}

/// What-If Simulation Advisor endpoint
async fn simulate_whatif(
    State(state): State<AppState>,
    Json(payload): Json<SimulateRequestDto>,
) -> Result<Json<SimulateResponseDto>, (StatusCode, String)> {
    let start = Instant::now();

    let (kb, pass_mark) = if let Some(path) = &payload.rules_path {
        load_knowledge_base_from_path(path)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    } else {
        let kb_guard = state.kb.read().await;
        let mark_guard = *state.pass_mark.read().await;
        (kb_guard.clone(), mark_guard)
    };

    let base_val = normalize_fact_value(payload.applicant);
    let base_facts = json_to_facts(&base_val);
    let base_report = evaluate_facts(&kb, &base_facts, pass_mark)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let current_score = base_report.total_score;
    let target_cutoff = payload.target_cutoff;
    let points_gap = (target_cutoff - current_score).max(0.0);

    let mut pathways = Vec::new();

    // Simulation 1: Provincial Nomination
    let mut pnp_val = base_val.clone();
    pnp_val["applicant"]["additional_factors"]["has_provincial_nomination"] =
        serde_json::Value::Bool(true);
    let pnp_facts = json_to_facts(&pnp_val);
    if let Ok(pnp_rep) = evaluate_facts(&kb, &pnp_facts, pass_mark) {
        let gain = pnp_rep.total_score - current_score;
        if gain > 0.0 {
            pathways.push(PathwayOptionDto {
                title: "Secure an Enhanced Provincial Nomination (e.g. Ontario Tech Draw)".to_string(),
                points_gain: gain,
                projected_total: pnp_rep.total_score,
                description: "Awarded +600 points, guaranteeing an Invitation to Apply (ITA) in the very next draw round.".to_string(),
                qualifies_for_draw: pnp_rep.total_score >= target_cutoff,
            });
        }
    }

    // Simulation 2: Language Exam Improvement to CLB 9+
    let mut lang_val = base_val.clone();
    lang_val["applicant"]["language"]["first_official"]["clb_reading"] = serde_json::json!(9);
    lang_val["applicant"]["language"]["first_official"]["clb_writing"] = serde_json::json!(9);
    lang_val["applicant"]["language"]["first_official"]["clb_listening"] = serde_json::json!(9);
    lang_val["applicant"]["language"]["first_official"]["clb_speaking"] = serde_json::json!(9);
    let lang_facts = json_to_facts(&lang_val);
    if let Ok(lang_rep) = evaluate_facts(&kb, &lang_facts, pass_mark) {
        let gain = lang_rep.total_score - current_score;
        if gain > 0.0 {
            pathways.push(PathwayOptionDto {
                title: "Retake Language Exam to reach CLB 9+ across all 4 abilities".to_string(),
                points_gain: gain,
                projected_total: lang_rep.total_score,
                description: "Unlocks maximum Skill Transferability multipliers for Education + Foreign Work.".to_string(),
                qualifies_for_draw: lang_rep.total_score >= target_cutoff,
            });
        }
    }

    // Simulation 3: Additional Canadian Domestic Work Experience
    let mut work_val = base_val.clone();
    let curr_work = work_val["applicant"]["work_experience"]["domestic_years"]
        .as_i64()
        .unwrap_or(0);
    work_val["applicant"]["work_experience"]["domestic_years"] = serde_json::json!(curr_work + 1);
    let work_facts = json_to_facts(&work_val);
    if let Ok(work_rep) = evaluate_facts(&kb, &work_facts, pass_mark) {
        let gain = work_rep.total_score - current_score;
        if gain > 0.0 {
            pathways.push(PathwayOptionDto {
                title: format!(
                    "Complete {} year(s) of Canadian Domestic Work Experience",
                    curr_work + 1
                ),
                points_gain: gain,
                projected_total: work_rep.total_score,
                description:
                    "Increases Core Human Capital domestic work points and boosts Transferability."
                        .to_string(),
                qualifies_for_draw: work_rep.total_score >= target_cutoff,
            });
        }
    }

    // Simulation 4: Sibling in Canada
    let mut sib_val = base_val.clone();
    sib_val["applicant"]["additional_factors"]["has_sibling_in_canada"] =
        serde_json::Value::Bool(true);
    let sib_facts = json_to_facts(&sib_val);
    if let Ok(sib_rep) = evaluate_facts(&kb, &sib_facts, pass_mark) {
        let gain = sib_rep.total_score - current_score;
        if gain > 0.0 {
            pathways.push(PathwayOptionDto {
                title: "Claim Canadian Citizen / Permanent Resident Sibling Bonus".to_string(),
                points_gain: gain,
                projected_total: sib_rep.total_score,
                description: "Provides an instant direct regulatory bonus.".to_string(),
                qualifies_for_draw: sib_rep.total_score >= target_cutoff,
            });
        }
    }

    let latency = start.elapsed().as_secs_f64() * 1_000_000.0;

    Ok(Json(SimulateResponseDto {
        candidate_id: base_report.applicant_id,
        current_score,
        target_cutoff,
        currently_qualifies: current_score >= target_cutoff,
        points_gap,
        pathways,
        latency_micros: latency,
    }))
}
