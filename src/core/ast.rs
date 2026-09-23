use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Comparison operators for condition evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    NotIn,
    Between,
    Contains,
    MatchesRegex,
    Exists,
    DoesNotExist,
}

/// A node in the Condition tree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Direct field comparison against a target value
    Compare {
        path: String,
        op: ComparisonOperator,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<Value>,
    },
    /// Logical AND: all child conditions must pass
    All { conditions: Vec<Condition> },
    /// Logical OR: at least one child condition must pass
    Any { conditions: Vec<Condition> },
    /// Logical NOR: no child conditions must pass
    None { conditions: Vec<Condition> },
    /// Logical NOT: inverts child condition
    Not { condition: Box<Condition> },
    /// MinCount: at least `min_required` child conditions must pass
    MinCount {
        min_required: usize,
        conditions: Vec<Condition>,
    },
    /// Field presence check
    Exists { path: String },
    /// Always evaluate to true
    Always,
    /// Temporal CEP Window evaluation (Drools Fusion equivalent)
    Temporal {
        field_path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reference_date_path: Option<String>,
        window_days: i64,
        #[serde(default = "default_direction")]
        direction: String, // "within_past", "within_future"
    },
}

fn default_direction() -> String {
    "within_past".to_string()
}

/// Method for calculating points in an action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PointsFormula {
    /// Fixed constant points
    Fixed { points: f64 },

    /// Linear scaling based on a numeric field value (e.g. years * factor)
    Scaled {
        path: String,
        factor: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },

    /// Single-dimension map lookup (e.g. highest_degree -> points, or age -> points)
    Lookup {
        path: String,
        mapping: HashMap<String, f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default: Option<f64>,
    },

    /// Two-dimensional matrix lookup (e.g. Education Level x CLB Language Band)
    MatrixLookup {
        row_path: String,
        col_path: String,
        /// row_value -> col_value -> points
        matrix: HashMap<String, HashMap<String, f64>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default: Option<f64>,
    },

    /// Official Language CLB Band Scorer for 4 abilities (reading, writing, listening, speaking)
    LanguageClbBand {
        /// Map of CLB level (e.g. "4".."10") to points per ability
        band_scores: HashMap<String, f64>,
        reading_path: String,
        writing_path: String,
        listening_path: String,
        speaking_path: String,
    },
}

/// Action to execute when a rule's condition is satisfied
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Award points towards a specific score category
    AwardPoints {
        category: String,
        formula: PointsFormula,
        #[serde(default)]
        reason: String,
    },
    /// Set the overall eligibility state
    SetEligibility {
        eligible: bool,
        reason: String,
    },
    /// Add a diagnostic tag or program stream flag (e.g. "stem_priority", "french_speaker")
    AddTag { tag: String },
    /// Enrich context with a computed attribute (Working Memory fact inference)
    SetAttribute { key: String, value: Value },
    /// Remove an attribute from context
    RemoveAttribute { key: String },
    /// Collection Accumulator (Drools accumulate)
    Accumulate {
        source_array: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<Condition>,
        function: String, // "sum", "sum_fte_years", "min", "max", "count"
        #[serde(default, skip_serializing_if = "Option::is_none")]
        field: Option<String>,
        target_attribute: String,
    },
}

/// Individual Rule definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    /// Unique identifier for the rule (e.g. "crs_age_points_single")
    pub id: String,
    /// Human readable name
    pub name: String,
    /// Detailed description of the regulation or policy
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Execution Phase / Agenda Group (e.g. "validation", "enrichment", "scoring", "capping")
    #[serde(default = "default_phase")]
    pub phase: String,
    /// Category grouping (e.g. "core_human_capital", "spouse_factors", "skill_transferability")
    #[serde(default = "default_category")]
    pub category: String,
    /// Evaluation priority / salience (higher numbers evaluate first)
    #[serde(default)]
    pub priority: i32,
    /// Activation Group name for XOR mutual exclusion (only first matching rule in group fires)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activation_group: Option<String>,
    /// Prevent self-reactivation loop when modifying context
    #[serde(default)]
    pub no_loop: bool,
    /// If true and condition fails or sets eligible=false, triggers early rejection
    #[serde(default)]
    pub is_eligibility_gate: bool,
    /// Whether this rule is currently enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Condition predicate tree
    pub condition: Condition,
    /// Actions executed when condition passes
    pub actions: Vec<Action>,
}

fn default_phase() -> String {
    "scoring".to_string()
}

fn default_category() -> String {
    "general".to_string()
}

fn default_enabled() -> bool {
    true
}

/// Category configuration including maximum point caps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryConfig {
    pub name: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_points: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Complete Rule Program / Suite
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleProgram {
    /// Program ID (e.g. "canada_crs_express_entry", "australia_gsm_189")
    pub id: String,
    /// Program display title
    pub name: String,
    /// Version string of the rule set / law regulation
    pub version: String,
    /// Detailed description of the immigration stream
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Category definitions and point caps
    #[serde(default)]
    pub categories: HashMap<String, CategoryConfig>,
    /// Maximum overall total points (e.g. 1200 for Canada CRS)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_points_cap: Option<f64>,
    /// Minimum passing score threshold (e.g. 65 for Australia Subclass 189, 70 for UK Skilled Worker)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass_mark_threshold: Option<f64>,
    /// List of rules to execute
    pub rules: Vec<Rule>,
}

impl RuleProgram {
    /// Parse a RuleProgram from YAML string
    pub fn from_yaml_str(yaml_str: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml_str)
    }

    /// Parse a RuleProgram from a YAML file
    pub fn from_yaml_file(path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let program = serde_yaml::from_str(&content)?;
        Ok(program)
    }

    /// Parse a RuleProgram from JSON string
    pub fn from_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}
