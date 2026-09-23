use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use rust_rules_engine::{Engine, FactContext, RuleProgram};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "rules-engine-cli",
    about = "A high-performance, deterministic Rules Engine in Rust for Merit-Based Immigration and Regulatory Systems",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate a single applicant profile against a folder of declarative rules
    Evaluate {
        /// Path to the rules folder / directory (or single rule file)
        #[arg(short, long, alias = "rules-dir")]
        rules: PathBuf,

        /// Path to the applicant profile YAML/JSON file
        #[arg(short, long)]
        applicant: PathBuf,

        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },

    /// Batch evaluate and rank all applicants in a directory against a folder of rules
    Batch {
        /// Path to the rules folder / directory (or single rule file)
        #[arg(short, long, alias = "rules-dir")]
        rules: PathBuf,

        /// Directory containing applicant profile YAML/JSON files
        #[arg(short, long)]
        applicants_dir: PathBuf,

        /// Minimum cutoff score filter (e.g. for Express Entry invitation draws)
        #[arg(short, long)]
        cutoff: Option<f64>,
    },

    /// Inspect a declarative rules folder (metadata, categories, caps, phases, and all loaded rules)
    Inspect {
        /// Path to the rules folder / directory (or single rule file)
        #[arg(short, long, alias = "rules-dir")]
        rules: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Evaluate {
            rules,
            applicant,
            format,
        } => {
            let program = load_rule_program(&rules)?;
            let context = load_fact_context(&applicant)?;

            let engine = Engine::new(program);
            let report = engine.evaluate(&context)?;

            match format {
                OutputFormat::Table => {
                    report.print_audit_table();
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                OutputFormat::Yaml => {
                    println!("{}", serde_yaml::to_string(&report)?);
                }
            }
        }

        Commands::Batch {
            rules,
            applicants_dir,
            cutoff,
        } => {
            let program = load_rule_program(&rules)?;
            let engine = Engine::new(program);

            let mut results = Vec::new();

            for entry in fs::read_dir(&applicants_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                    || path.extension().and_then(|e| e.to_str()) == Some("json")
                {
                    if let Ok(ctx) = load_fact_context(&path) {
                        if let Ok(report) = engine.evaluate(&ctx) {
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            results.push((file_name, report));
                        }
                    }
                }
            }

            // Sort by total score descending (Ranking)
            results.sort_by(|a, b| b.1.total_score.partial_cmp(&a.1.total_score).unwrap_or(std::cmp::Ordering::Equal));

            println!("\n{}", format!(" 🏆 BATCH RANKING & SELECTION DRAW: {} ", engine.program().name).bold().on_purple().white());
            if let Some(cut) = cutoff {
                println!("  Cutoff Score Threshold: {}", format!("{:.1} points", cut).yellow().bold());
            }
            println!("  Total Candidates Evaluated: {}", results.len());

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Rank"),
                    Cell::new("Candidate ID / File"),
                    Cell::new("Total Score"),
                    Cell::new("Status"),
                    Cell::new("Draw Result"),
                ]);

            for (idx, (file, report)) in results.iter().enumerate() {
                let rank = (idx + 1).to_string();
                let candidate_id = report.applicant_id.clone().unwrap_or_else(|| file.clone());
                let score_str = format!("{:.1}", report.total_score);

                let (status_str, draw_result) = if !report.is_eligible() {
                    (Cell::new("Ineligible").fg(Color::Red), Cell::new("Disqualified").fg(Color::Red))
                } else if let Some(cut) = cutoff {
                    if report.total_score >= cut {
                        (Cell::new("Eligible").fg(Color::Green), Cell::new("SELECTED (ITA)").fg(Color::Green))
                    } else {
                        (Cell::new("Eligible").fg(Color::Green), Cell::new("Below Cutoff").fg(Color::Yellow))
                    }
                } else {
                    (Cell::new("Eligible").fg(Color::Green), Cell::new("Ranked").fg(Color::Cyan))
                };

                table.add_row(vec![
                    Cell::new(rank),
                    Cell::new(candidate_id),
                    Cell::new(score_str).fg(Color::Cyan),
                    status_str,
                    draw_result,
                ]);
            }

            println!("{table}\n");
        }

        Commands::Inspect { rules } => {
            let program = load_rule_program(&rules)?;
            println!("\n{}", format!(" 🔍 PROGRAM INSPECTION: {} (v{}) ", program.name, program.version).bold().on_cyan().black());
            if let Some(desc) = &program.description {
                println!("Description: {}", desc.dimmed());
            }
            if let Some(pass) = program.pass_mark_threshold {
                println!("Pass Mark Threshold: {}", format!("{:.1} points", pass).yellow().bold());
            }
            if let Some(total_cap) = program.total_points_cap {
                println!("Overall Total Cap: {}", format!("{:.1} points", total_cap).yellow().bold());
            }

            println!("\n{}", "📊 Category Budgets & Sub-Caps:".bold().underline());
            for (_key, cat) in &program.categories {
                let cap_str = cat.max_points.map(|p| format!("{:.1} pts", p)).unwrap_or_else(|| "No Cap".to_string());
                println!("  • {:<25} -> Max: {}", cat.display_name.cyan(), cap_str.yellow());
            }

            println!("\n{}", "📜 Defined Rules by Phase:".bold().underline());
            let mut rules_by_phase: std::collections::BTreeMap<String, Vec<&rust_rules_engine::Rule>> = std::collections::BTreeMap::new();
            for r in &program.rules {
                rules_by_phase.entry(r.phase.clone()).or_default().push(r);
            }

            for (phase, rules) in rules_by_phase {
                println!("\n  [{}] ({} rules)", phase.to_uppercase().magenta().bold(), rules.len());
                for r in rules {
                    let gate_tag = if r.is_eligibility_gate { "[GATE]".red() } else { "".normal() };
                    let group_tag = r.activation_group.as_ref().map(|g| format!("[XOR: {}]", g).yellow()).unwrap_or_default();
                    println!("    • {:<35} (Priority: {:>4}) {} {}", r.name, r.priority, gate_tag, group_tag);
                }
            }
            println!();
        }
    }

    Ok(())
}

fn load_rule_program(path: &Path) -> Result<RuleProgram, Box<dyn std::error::Error>> {
    RuleProgram::from_path(path)
}

fn load_fact_context(path: &Path) -> Result<FactContext, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let val: serde_json::Value = if path.extension().and_then(|e| e.to_str()) == Some("json") {
        serde_json::from_str(&content)?
    } else {
        serde_yaml::from_str(&content)?
    };
    Ok(FactContext::from_value(val))
}
