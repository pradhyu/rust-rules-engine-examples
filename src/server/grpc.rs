use std::time::Instant;
use tonic::{Request, Response, Status};

use crate::core::{Engine, FactContext, RuleProgram};
use crate::server::rest::AppState;

pub mod proto {
    tonic::include_proto!("rules_engine");
}

use proto::rules_service_server::RulesService;
pub use proto::rules_service_server::RulesServiceServer;
use proto::{
    BatchEvaluateRequest, BatchEvaluateResponse, CategoryBreakdown, EvaluateRequest,
    EvaluateResponse, FiredRule, InspectRulesRequest, InspectRulesResponse, PathwaySimulation,
    RankedCandidate, ReloadRulesRequest, ReloadRulesResponse, WhatIfRequest, WhatIfResponse,
};

/// Tonic gRPC Service Implementation for Rules Engine
#[derive(Clone)]
pub struct RulesGrpcService {
    state: AppState,
}

impl RulesGrpcService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

fn normalize_fact_value(val: serde_json::Value) -> serde_json::Value {
    if val.get("applicant").is_some() {
        val
    } else {
        serde_json::json!({ "applicant": val })
    }
}

#[tonic::async_trait]
#[allow(clippy::result_large_err)]
impl RulesService for RulesGrpcService {
    /// Evaluate a single applicant fact model in microsecond real-time
    async fn evaluate(
        &self,
        request: Request<EvaluateRequest>,
    ) -> Result<Response<EvaluateResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();

        let raw_val: serde_json::Value =
            serde_json::from_str(&req.applicant_json).map_err(|e| {
                Status::invalid_argument(format!("Invalid JSON applicant payload: {}", e))
            })?;

        let app_val = normalize_fact_value(raw_val);

        let report = if !req.rules_path.is_empty() {
            let program = if std::path::Path::new(&req.rules_path).is_dir() {
                RuleProgram::from_directory(&req.rules_path)
                    .map_err(|e| Status::invalid_argument(format!("Failed loading rules: {}", e)))?
            } else {
                RuleProgram::from_file(&req.rules_path)
                    .map_err(|e| Status::invalid_argument(format!("Failed loading rules: {}", e)))?
            };
            let engine = Engine::new(program);
            let ctx = FactContext::from_value(app_val);
            engine
                .evaluate(&ctx)
                .map_err(|e| Status::internal(format!("Evaluation error: {}", e)))?
        } else {
            let engine = self.state.engine.read().await;
            let ctx = FactContext::from_value(app_val);
            engine
                .evaluate(&ctx)
                .map_err(|e| Status::internal(format!("Evaluation error: {}", e)))?
        };

        let latency = start.elapsed().as_secs_f64() * 1_000_000.0;
        let is_eligible = report.is_eligible();

        let fired_rules = report
            .fired_rules
            .into_iter()
            .map(|r| FiredRule {
                rule_id: r.rule_id,
                rule_name: r.rule_name,
                category: r.category,
                points_awarded: r.points_awarded,
                reason: r.reason,
                phase: r.phase,
            })
            .collect();

        let categories = report
            .category_scores
            .into_iter()
            .map(|v| CategoryBreakdown {
                category_name: v.category,
                display_name: v.display_name,
                raw_score: v.raw_points,
                capped_score: v.capped_points,
                max_points: v.max_points.unwrap_or(0.0),
            })
            .collect();

        Ok(Response::new(EvaluateResponse {
            is_eligible,
            total_score: report.total_score,
            pass_mark_threshold: report.pass_mark_threshold.unwrap_or(0.0),
            ineligibility_reasons: report.ineligibility_reasons,
            fired_rules,
            categories,
            tags: report.tags,
            latency_micros: latency,
        }))
    }

    /// Batch candidate evaluation and invitation draw ranking
    async fn batch_evaluate(
        &self,
        request: Request<BatchEvaluateRequest>,
    ) -> Result<Response<BatchEvaluateResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        let engine = self.state.engine.read().await;

        let mut candidate_results = Vec::new();

        for (idx, app_str) in req.applicant_jsons.iter().enumerate() {
            if let Ok(raw_val) = serde_json::from_str::<serde_json::Value>(app_str) {
                let app_val = normalize_fact_value(raw_val);
                let ctx = FactContext::from_value(app_val);
                if let Ok(report) = engine.evaluate(&ctx) {
                    let candidate_id = report
                        .applicant_id
                        .clone()
                        .unwrap_or_else(|| format!("APP-{:04}", idx + 1));
                    candidate_results.push((candidate_id, report));
                }
            }
        }

        // Sort descending by score
        candidate_results.sort_by(|a, b| {
            b.1.total_score
                .partial_cmp(&a.1.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let candidates = candidate_results
            .into_iter()
            .enumerate()
            .map(|(rank_idx, (id, report))| {
                let draw_status = if !report.is_eligible() {
                    "Ineligible".to_string()
                } else if req.cutoff_score > 0.0 {
                    if report.total_score >= req.cutoff_score {
                        "SELECTED (ITA)".to_string()
                    } else {
                        "Below Cutoff".to_string()
                    }
                } else {
                    "Ranked".to_string()
                };

                RankedCandidate {
                    rank: (rank_idx + 1) as i32,
                    candidate_id: id,
                    total_score: report.total_score,
                    is_eligible: report.is_eligible(),
                    draw_status,
                }
            })
            .collect();

        let latency = start.elapsed().as_secs_f64() * 1_000_000.0;

        Ok(Response::new(BatchEvaluateResponse {
            candidates,
            total_evaluated: req.applicant_jsons.len() as i32,
            latency_micros: latency,
        }))
    }

    /// Run real-time counterfactual What-If pathway simulations
    async fn simulate_what_if(
        &self,
        request: Request<WhatIfRequest>,
    ) -> Result<Response<WhatIfResponse>, Status> {
        let req = request.into_inner();
        let engine = self.state.engine.read().await;

        let raw_val: serde_json::Value = serde_json::from_str(&req.applicant_json)
            .map_err(|e| Status::invalid_argument(format!("Invalid JSON: {}", e)))?;

        let app_val = normalize_fact_value(raw_val);
        let base_ctx = FactContext::from_value(app_val.clone());
        let base_report = engine
            .evaluate(&base_ctx)
            .map_err(|e| Status::internal(format!("Evaluation error: {}", e)))?;

        let current_score = base_report.total_score;
        let target_cutoff = req.target_cutoff;
        let points_gap = (target_cutoff - current_score).max(0.0);

        let mut pathways = Vec::new();

        // 1. Provincial Nomination
        let mut pnp_val = app_val.clone();
        pnp_val["applicant"]["additional_factors"]["has_provincial_nomination"] =
            serde_json::Value::Bool(true);
        if let Ok(pnp_rep) = engine.evaluate(&FactContext::from_value(pnp_val)) {
            let gain = pnp_rep.total_score - current_score;
            if gain > 0.0 {
                pathways.push(PathwaySimulation {
                    title: "Secure an Enhanced Provincial Nomination (e.g. Ontario Tech Draw)"
                        .to_string(),
                    points_gain: gain,
                    projected_total: pnp_rep.total_score,
                    description: "Awarded +600 points, guaranteeing an ITA in the next draw round."
                        .to_string(),
                    qualifies_for_draw: pnp_rep.total_score >= target_cutoff,
                });
            }
        }

        // 2. Language Exam improvement to CLB 9+
        let mut lang_val = app_val.clone();
        lang_val["applicant"]["language"]["first_official"]["clb_reading"] = serde_json::json!(9);
        lang_val["applicant"]["language"]["first_official"]["clb_writing"] = serde_json::json!(9);
        lang_val["applicant"]["language"]["first_official"]["clb_listening"] = serde_json::json!(9);
        lang_val["applicant"]["language"]["first_official"]["clb_speaking"] = serde_json::json!(9);
        if let Ok(lang_rep) = engine.evaluate(&FactContext::from_value(lang_val)) {
            let gain = lang_rep.total_score - current_score;
            if gain > 0.0 {
                pathways.push(PathwaySimulation {
                    title: "Retake Language Exam to reach CLB 9+ across all 4 abilities"
                        .to_string(),
                    points_gain: gain,
                    projected_total: lang_rep.total_score,
                    description: "Unlocks maximum Skill Transferability multipliers.".to_string(),
                    qualifies_for_draw: lang_rep.total_score >= target_cutoff,
                });
            }
        }

        // 3. Extra Work Experience
        let mut work_val = app_val.clone();
        let curr_work = work_val["applicant"]["work_experience"]["domestic_years"]
            .as_i64()
            .unwrap_or(0);
        work_val["applicant"]["work_experience"]["domestic_years"] =
            serde_json::json!(curr_work + 1);
        if let Ok(work_rep) = engine.evaluate(&FactContext::from_value(work_val)) {
            let gain = work_rep.total_score - current_score;
            if gain > 0.0 {
                pathways.push(PathwaySimulation {
                    title: format!(
                        "Complete {} year(s) of Canadian Domestic Work Experience",
                        curr_work + 1
                    ),
                    points_gain: gain,
                    projected_total: work_rep.total_score,
                    description: "Increases Core Human Capital domestic work points.".to_string(),
                    qualifies_for_draw: work_rep.total_score >= target_cutoff,
                });
            }
        }

        Ok(Response::new(WhatIfResponse {
            current_score,
            target_cutoff,
            currently_qualifies: current_score >= target_cutoff,
            points_gap,
            pathways,
        }))
    }

    /// Dynamic hot-reload of active ruleset
    async fn reload_rules(
        &self,
        request: Request<ReloadRulesRequest>,
    ) -> Result<Response<ReloadRulesResponse>, Status> {
        let req = request.into_inner();
        let program = RuleProgram::from_path(&req.rules_path).map_err(|e| {
            Status::invalid_argument(format!(
                "Failed to load rules from '{}': {}",
                req.rules_path, e
            ))
        })?;

        let ruleset_id = program.id.clone();
        let ruleset_name = program.name.clone();
        let version = program.version.clone();
        let rule_count = program.rules.len() as i32;

        {
            let mut engine_guard = self.state.engine.write().await;
            *engine_guard = Engine::new(program);
        }
        {
            let mut name_guard = self.state.default_rules_name.write().await;
            *name_guard = ruleset_name.clone();
        }

        Ok(Response::new(ReloadRulesResponse {
            success: true,
            message: format!(
                "Successfully loaded ruleset '{}' from '{}'",
                ruleset_name, req.rules_path
            ),
            ruleset_id,
            ruleset_name,
            version,
            rule_count,
        }))
    }

    /// Inspect active ruleset metadata and category caps
    async fn inspect_rules(
        &self,
        _request: Request<InspectRulesRequest>,
    ) -> Result<Response<InspectRulesResponse>, Status> {
        let engine = self.state.engine.read().await;
        let p = engine.program();

        let categories = p
            .categories
            .values()
            .map(|c| CategoryBreakdown {
                category_name: c.name.clone(),
                display_name: c.display_name.clone(),
                raw_score: 0.0,
                capped_score: 0.0,
                max_points: c.max_points.unwrap_or(0.0),
            })
            .collect();

        Ok(Response::new(InspectRulesResponse {
            id: p.id.clone(),
            name: p.name.clone(),
            version: p.version.clone(),
            description: p.description.clone().unwrap_or_default(),
            pass_mark_threshold: p.pass_mark_threshold.unwrap_or(0.0),
            total_points_cap: p.total_points_cap.unwrap_or(0.0),
            rule_count: p.rules.len() as i32,
            categories,
        }))
    }
}
