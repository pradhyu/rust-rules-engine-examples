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

    /// Multi-Column Decision Table with DMN / Drools Hit Policies
    DecisionTable(Box<DecisionTable>),
}

/// DMN & Drools-equivalent Hit Policies for Decision Tables
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum HitPolicy {
    #[default]
    First,       // First matching row returns output
    Unique,      // Exactly one row must match
    Priority,    // Highest priority row
    Any,         // Multiple matching rows with identical output
    CollectSum,  // Sum of all matching row outputs (C+)
    CollectMin,  // Minimum of all matching row outputs (C<)
    CollectMax,  // Maximum of all matching row outputs (C>)
    CollectCount,// Count of matching rows (C#)
    RuleOrder,   // Ordered list of all matching outputs
}

/// Input column specification for a Decision Table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionTableInput {
    pub name: String,
    pub path: String,
}

/// Output column specification for a Decision Table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionTableOutput {
    pub name: String,
    pub category: String,
}

/// A single row in a Decision Table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionTableRow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Expressions for each input column (e.g. ">= 9", "in ['master', 'doctorate']", "-")
    pub input_entries: Vec<String>,
    /// Outputs for each output column (e.g. 50.0)
    pub output_entries: Vec<Value>,
}

/// Multi-Column Decision Table Definition (Drools / DMN Standard)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionTable {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub hit_policy: HitPolicy,
    pub inputs: Vec<DecisionTableInput>,
    pub outputs: Vec<DecisionTableOutput>,
    pub rows: Vec<DecisionTableRow>,
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_bracket: usize = 0;
    let mut in_paren: usize = 0;
    let mut in_quote = false;
    let mut quote_char = '"';

    for c in line.chars() {
        match c {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = c;
                current.push(c);
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
                current.push(c);
            }
            '[' if !in_quote => {
                in_bracket += 1;
                current.push(c);
            }
            ']' if !in_quote => {
                in_bracket = in_bracket.saturating_sub(1);
                current.push(c);
            }
            '(' if !in_quote => {
                in_paren += 1;
                current.push(c);
            }
            ')' if !in_quote => {
                in_paren = in_paren.saturating_sub(1);
                current.push(c);
            }
            ',' if !in_quote && in_bracket == 0 && in_paren == 0 => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

impl DecisionTable {
    /// Parse a DecisionTable from CSV string (Spreadsheet decision table)
    pub fn from_csv_str(csv_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut lines = csv_str.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'));
        
        let header_line = lines.next().ok_or("CSV Decision Table missing header line")?;
        let headers = split_csv_line(header_line);
        
        if headers.len() < 2 {
            return Err("CSV Decision Table must have at least 1 input column and 1 output column".into());
        }

        let num_inputs = headers.len() - 2; // last 2 are output_pts and description
        let mut inputs = Vec::new();
        for h in &headers[0..num_inputs] {
            inputs.push(DecisionTableInput {
                name: h.to_string(),
                path: h.to_string(),
            });
        }

        let output_name = &headers[num_inputs];
        let outputs = vec![DecisionTableOutput {
            name: "points".to_string(),
            category: output_name.to_string(),
        }];

        let mut rows = Vec::new();
        for (idx, line) in lines.enumerate() {
            let cells = split_csv_line(line);
            if cells.len() >= headers.len() {
                let input_entries: Vec<String> = cells[0..num_inputs].to_vec();
                let out_val: f64 = cells[num_inputs].parse().unwrap_or(0.0);
                let desc = if cells.len() > num_inputs + 1 {
                    Some(cells[num_inputs + 1].clone())
                } else {
                    None
                };

                rows.push(DecisionTableRow {
                    id: Some(format!("row_{}", idx + 1)),
                    description: desc,
                    input_entries,
                    output_entries: vec![Value::from(out_val)],
                });
            }
        }

        Ok(DecisionTable {
            id: "csv_decision_table".to_string(),
            name: "CSV Decision Table".to_string(),
            hit_policy: HitPolicy::CollectSum,
            inputs,
            outputs,
            rows,
        })
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RuleListFragment {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub categories: HashMap<String, CategoryConfig>,
    #[serde(default)]
    pub total_points_cap: Option<f64>,
    #[serde(default)]
    pub pass_mark_threshold: Option<f64>,
    #[serde(default)]
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

    /// Parse a RuleProgram from a file path (JSON or YAML)
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let p = path.as_ref();
        let content = std::fs::read_to_string(p)?;
        if p.extension().and_then(|e| e.to_str()) == Some("json") {
            Ok(Self::from_json_str(&content)?)
        } else {
            Ok(Self::from_yaml_str(&content)?)
        }
    }

    /// Load and merge ALL rule files from a directory into a single unified RuleProgram
    pub fn from_directory(dir_path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = dir_path.as_ref();
        if !dir.is_dir() {
            return Err(format!("Path is not a directory: {}", dir.display()).into());
        }

        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .collect();
        
        // Sort filenames for deterministic load order
        entries.sort_by_key(|a| a.file_name());

        let mut unified = RuleProgram {
            id: String::new(),
            name: String::new(),
            version: "1.0.0".to_string(),
            description: None,
            categories: HashMap::new(),
            total_points_cap: None,
            pass_mark_threshold: None,
            rules: Vec::new(),
        };

        let mut loaded_files = 0;

        for entry in entries {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "yaml" || ext == "yml" || ext == "json" {
                    let content = std::fs::read_to_string(&path)?;
                    let fragment: Result<RuleListFragment, Box<dyn std::error::Error>> = if ext == "json" {
                        serde_json::from_str(&content).map_err(|e| e.into())
                    } else {
                        serde_yaml::from_str(&content).map_err(|e| e.into())
                    };

                    if let Ok(frag) = fragment {
                        if unified.id.is_empty() {
                            if let Some(id) = frag.id {
                                unified.id = id;
                            }
                        }
                        if unified.name.is_empty() {
                            if let Some(name) = frag.name {
                                unified.name = name;
                            }
                        }
                        if let Some(ver) = frag.version {
                            unified.version = ver;
                        }
                        if frag.description.is_some() && unified.description.is_none() {
                            unified.description = frag.description;
                        }
                        if frag.total_points_cap.is_some() && unified.total_points_cap.is_none() {
                            unified.total_points_cap = frag.total_points_cap;
                        }
                        if frag.pass_mark_threshold.is_some() && unified.pass_mark_threshold.is_none() {
                            unified.pass_mark_threshold = frag.pass_mark_threshold;
                        }

                        // Merge categories
                        for (k, v) in frag.categories {
                            unified.categories.insert(k, v);
                        }

                        // Append rules
                        for r in frag.rules {
                            unified.rules.push(r);
                        }

                        loaded_files += 1;
                    }
                }
            }
        }

        if loaded_files == 0 {
            return Err(format!("No valid rule files (.yaml, .yml, .json) found in directory: {}", dir.display()).into());
        }

        if unified.id.is_empty() {
            unified.id = dir.file_name().unwrap_or_default().to_string_lossy().to_string();
        }
        if unified.name.is_empty() {
            unified.name = format!("Modular Rule Program ({})", unified.id);
        }

        Ok(unified)
    }

    /// Automatically load from either a file or a directory of rule files
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let p = path.as_ref();
        if p.is_dir() {
            Self::from_directory(p)
        } else {
            Self::from_file(p)
        }
    }
}

