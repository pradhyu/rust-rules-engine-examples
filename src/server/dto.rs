//! Request and Response Data Transfer Objects (DTOs)
//!
//! This module explicitly separates all network data models used across
//! REST and gRPC API boundaries from internal engine AST structs.

use crate::evaluator::{CategoryConfig, CategoryScoreBreakdown, FiredRuleRecord};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// 1. SINGLE CANDIDATE EVALUATION DATA MODELS
// =============================================================================

/// Request payload for evaluating a single candidate against rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateRequestDto {
    /// Optional path or directory to rules (defaults to the currently loaded server ruleset)
    #[serde(default)]
    pub rules_path: Option<String>,

    /// The JSON fact payload containing applicant attributes (age, education, work, etc.)
    pub applicant: serde_json::Value,
}

/// Response returned by real-time single evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateResponseDto {
    /// Unique candidate identifier if provided in facts
    pub applicant_id: Option<String>,

    /// Overall qualification decision (true if meets criteria and passes threshold)
    pub eligible: bool,

    /// Total calculated points score
    pub total_score: f64,

    /// Minimum passing score threshold configured in rules
    pub pass_mark_threshold: Option<f64>,

    /// List of gate rejection reasons if disqualified
    pub ineligibility_reasons: Vec<String>,

    /// Itemized trace of all rules that fired
    pub fired_rules: Vec<FiredRuleRecord>,

    /// Points breakdown per regulatory category (raw vs capped)
    pub category_scores: Vec<CategoryScoreBreakdown>,

    /// Inferred diagnostic tags (e.g. "stem_priority_candidate", "french_speaker")
    pub tags: Vec<String>,

    /// Engine execution latency in microseconds (µs)
    pub latency_micros: f64,
}

// =============================================================================
// 2. BATCH EVALUATION & RANKING DATA MODELS
// =============================================================================

/// Request payload for batch evaluating and ranking multiple candidates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestDto {
    /// Optional custom rules path
    #[serde(default)]
    pub rules_path: Option<String>,

    /// Array of candidate fact JSON objects
    pub applicants: Vec<serde_json::Value>,

    /// Minimum cutoff score filter (e.g. 500 for Express Entry invitation draw)
    #[serde(default)]
    pub cutoff: Option<f64>,
}

/// An individual candidate's ranking position in a batch draw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedCandidateDto {
    pub rank: usize,
    pub candidate_id: String,
    pub total_score: f64,
    pub eligible: bool,
    pub draw_result: String,
}

/// Response returned by batch evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponseDto {
    pub ruleset_name: String,
    pub total_evaluated: usize,
    pub cutoff: Option<f64>,
    pub candidates: Vec<RankedCandidateDto>,
    pub latency_micros: f64,
}

// =============================================================================
// 3. WHAT-IF COUNTERFACTUAL SIMULATION DATA MODELS
// =============================================================================

/// Request payload for running What-If scenario simulations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateRequestDto {
    /// Optional custom rules path
    #[serde(default)]
    pub rules_path: Option<String>,

    /// Baseline applicant fact payload
    pub applicant: serde_json::Value,

    /// Target cutoff score the candidate wishes to achieve (e.g. 485.0)
    #[serde(default = "default_cutoff")]
    pub target_cutoff: f64,
}

fn default_cutoff() -> f64 {
    485.0
}

/// An actionable pathway recommendation returned by simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathwayOptionDto {
    pub title: String,
    pub points_gain: f64,
    pub projected_total: f64,
    pub description: String,
    pub qualifies_for_draw: bool,
}

/// Response containing current score and projected improvement pathways
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateResponseDto {
    pub candidate_id: Option<String>,
    pub current_score: f64,
    pub target_cutoff: f64,
    pub currently_qualifies: bool,
    pub points_gap: f64,
    pub pathways: Vec<PathwayOptionDto>,
    pub latency_micros: f64,
}

// =============================================================================
// 4. HEALTH CHECK & RULESET INSPECTION DATA MODELS
// =============================================================================

/// Health check response model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponseDto {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: String,
}

/// Ruleset inspection metadata response model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesetInspectionDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub pass_mark_threshold: Option<f64>,
    pub total_points_cap: Option<f64>,
    pub categories: HashMap<String, CategoryConfig>,
    pub rule_count: usize,
}

// =============================================================================
// 5. DYNAMIC RULESET HOT-RELOAD DATA MODELS
// =============================================================================

/// Request to reload or switch the active server ruleset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadRulesRequestDto {
    /// Path or directory of the new rule program to load
    pub rules_path: String,
}

/// Response confirming ruleset reload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadRulesResponseDto {
    pub success: bool,
    pub message: String,
    pub ruleset_id: String,
    pub ruleset_name: String,
    pub version: String,
    pub rule_count: usize,
}
