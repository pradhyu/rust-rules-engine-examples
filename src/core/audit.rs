use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScoreBreakdown {
    pub category: String,
    pub display_name: String,
    pub raw_points: f64,
    pub capped_points: f64,
    pub max_points: Option<f64>,
    pub is_capped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiredRuleRecord {
    pub rule_id: String,
    pub rule_name: String,
    pub category: String,
    pub points_awarded: f64,
    pub reason: String,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub program_id: String,
    pub program_name: String,
    pub version: String,
    pub applicant_id: Option<String>,
    pub eligible: bool,
    pub total_score: f64,
    pub pass_mark_threshold: Option<f64>,
    pub passed_threshold: bool,
    pub category_scores: Vec<CategoryScoreBreakdown>,
    pub fired_rules: Vec<FiredRuleRecord>,
    pub ineligibility_reasons: Vec<String>,
    pub tags: Vec<String>,
}

impl AuditReport {
    pub fn is_eligible(&self) -> bool {
        self.eligible && self.passed_threshold
    }

    /// Output a formatted audit table to stdout
    pub fn print_audit_table(&self) {
        println!();
        let title = format!(" ⚖️  DECISION AUDIT REPORT: {} (v{}) ", self.program_name, self.version);
        println!("{}", title.bold().on_blue().white());
        if let Some(ref id) = self.applicant_id {
            println!("  Applicant ID: {}", id.cyan().bold());
        }

        let status_str = if self.is_eligible() {
            " QUALIFIED / ELIGIBLE ".bold().on_green().white()
        } else {
            " INELIGIBLE / REJECTED ".bold().on_red().white()
        };
        println!("  Overall Status: {status_str}");
        println!("  Total Calculated Score: {} points", format!("{:.1}", self.total_score).yellow().bold());
        if let Some(threshold) = self.pass_mark_threshold {
            let pass_str = if self.passed_threshold { "PASSED".green() } else { "FAILED".red() };
            println!("  Pass Mark Threshold: {:.1} points ({})", threshold, pass_str);
        }

        if !self.tags.is_empty() {
            println!("  Tags & Badges: {}", self.tags.join(", ").magenta());
        }

        if !self.ineligibility_reasons.is_empty() {
            println!("\n{}", "⚠️  INELIGIBILITY / GATE BLOCKERS:".red().bold());
            for reason in &self.ineligibility_reasons {
                println!("  • {}", reason.red());
            }
        }

        // Category Breakdown Table
        println!("\n{}", "📊 Category Score Breakdown:".bold().underline());
        let mut cat_table = Table::new();
        cat_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Category"),
                Cell::new("Raw Points"),
                Cell::new("Capped Points"),
                Cell::new("Max Allowed"),
                Cell::new("Status"),
            ]);

        for cat in &self.category_scores {
            let max_str = cat.max_points.map(|m| format!("{:.1}", m)).unwrap_or_else(|| "No Cap".to_string());
            let status_cell = if cat.is_capped {
                Cell::new("CAPPED (Overflow Truncated)").fg(Color::Yellow)
            } else {
                Cell::new("OK").fg(Color::Green)
            };

            cat_table.add_row(vec![
                Cell::new(&cat.display_name),
                Cell::new(format!("{:.1}", cat.raw_points)),
                Cell::new(format!("{:.1}", cat.capped_points)).fg(Color::Cyan),
                Cell::new(max_str),
                status_cell,
            ]);
        }
        println!("{cat_table}");

        // Itemized Fired Rules
        println!("\n{}", "📜 Itemized Fired Rules Trace:".bold().underline());
        let mut rule_table = Table::new();
        rule_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Rule ID"),
                Cell::new("Rule Name"),
                Cell::new("Phase"),
                Cell::new("Points"),
                Cell::new("Justification / Audit Note"),
            ]);

        for rule in &self.fired_rules {
            let pts_str = if rule.points_awarded > 0.0 {
                format!("+{:.1}", rule.points_awarded)
            } else {
                "0.0".to_string()
            };

            rule_table.add_row(vec![
                Cell::new(&rule.rule_id).fg(Color::Cyan),
                Cell::new(&rule.rule_name),
                Cell::new(&rule.phase),
                Cell::new(pts_str).fg(Color::Green),
                Cell::new(&rule.reason),
            ]);
        }
        println!("{rule_table}\n");
    }
}
