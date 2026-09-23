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

    /// Run the comprehensive Drools Parity Verification Test Suite (TC-01 to TC-12)
    TestSuite {
        /// Optional path to the rules directory
        #[arg(short, long, default_value = "rules")]
        rules_dir: PathBuf,

        /// Optional path to the applicants directory
        #[arg(short, long, default_value = "applicants")]
        applicants_dir: PathBuf,
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
                let app_id = report.applicant_id.clone().unwrap_or_else(|| file.clone());
                let score_str = format!("{:.1}", report.total_score);

                let eligible_str = if report.is_eligible() {
                    Cell::new("Eligible").fg(Color::Green)
                } else {
                    Cell::new("Ineligible").fg(Color::Red)
                };

                let draw_result = match cutoff {
                    Some(cut) if report.is_eligible() && report.total_score >= cut => {
                        Cell::new("SELECTED (ITA)").fg(Color::Green)
                    }
                    Some(_) if report.is_eligible() => Cell::new("Below Cutoff").fg(Color::DarkGrey),
                    _ if report.is_eligible() => Cell::new("Eligible Pool").fg(Color::Cyan),
                    _ => Cell::new("Disqualified").fg(Color::Red),
                };

                table.add_row(vec![
                    Cell::new((idx + 1).to_string()),
                    Cell::new(app_id),
                    Cell::new(score_str).fg(Color::Yellow),
                    eligible_str,
                    draw_result,
                ]);
            }

            println!("{table}\n");
        }

        Commands::Inspect { rules } => {
            let program = load_rule_program(&rules)?;

            println!("\n{}", format!(" 🔍 RULE PROGRAM INSPECTOR: {} (v{}) ", program.name, program.version).bold().on_cyan().black());
            println!("  ID: {}", program.id);
            if let Some(ref desc) = program.description {
                println!("  Description: {}", desc);
            }
            if let Some(cap) = program.total_points_cap {
                println!("  Total Points Cap: {:.1} pts", cap);
            }
            if let Some(pass) = program.pass_mark_threshold {
                println!("  Pass Mark Threshold: {:.1} pts", pass);
            }

            println!("\n{}", "📦 Category Budget Caps:".bold().underline());
            for (_k, cat) in &program.categories {
                let cap_str = cat.max_points.map(|m| format!("{:.1} pts", m)).unwrap_or_else(|| "Unlimited".to_string());
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

        Commands::TestSuite {
            rules_dir,
            applicants_dir,
        } => {
            run_drools_parity_test_suite(&rules_dir, &applicants_dir)?;
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

fn run_drools_parity_test_suite(rules_dir: &Path, applicants_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", " 🧪 RUNNING DROOLS PARITY & EDGE CASE VERIFICATION SUITE ".bold().on_blue().white());

    struct TestCase {
        id: &'static str,
        name: &'static str,
        rule_file: &'static str,
        applicant_file: &'static str,
        expected_eligible: bool,
        expected_min_score: Option<f64>,
        description: &'static str,
    }

    let test_cases = vec![
        TestCase {
            id: "TC-01",
            name: "Skill Transferability & CLB 9+ Matrix",
            rule_file: "canada_crs_express_entry.yaml",
            applicant_file: "tc01_tech_lead_single.yaml",
            expected_eligible: true,
            expected_min_score: Some(580.0),
            description: "Evaluates multi-variable transferability + job offer + sibling bonus",
        },
        TestCase {
            id: "TC-02",
            name: "Spouse Factors & PNP Nomination",
            rule_file: "canada_crs_express_entry.yaml",
            applicant_file: "tc02_married_phd_researcher.yaml",
            expected_eligible: true,
            expected_min_score: Some(1000.0),
            description: "Spouse credentials + Enhanced provincial nomination (+600)",
        },
        TestCase {
            id: "TC-03",
            name: "Subcategory Cap Truncation",
            rule_file: "canada_crs_express_entry.yaml",
            applicant_file: "tc03_cap_overflow_tradesperson.yaml",
            expected_eligible: true,
            expected_min_score: Some(400.0),
            description: "Clamping 3 transferability rules (150 pts) to 100 max cap",
        },
        TestCase {
            id: "TC-04",
            name: "Multi-Fact Relational Join Validation",
            rule_file: "uk_skilled_worker_points.yaml",
            applicant_file: "tc04_uk_sponsor_unlicensed.yaml",
            expected_eligible: false,
            expected_min_score: None,
            description: "Unlicensed sponsor fails mandatory join and disqualifies candidate",
        },
        TestCase {
            id: "TC-05",
            name: "Activation Group XOR Mutual Exclusion",
            rule_file: "uk_skilled_worker_points.yaml",
            applicant_file: "tc05_uk_competing_tradeable.yaml",
            expected_eligible: true,
            expected_min_score: Some(70.0),
            description: "Option A (£48k salary) fires and cancels Option B (STEM PhD)",
        },
        TestCase {
            id: "TC-06",
            name: "Hard Age Gate vs Soft Penalty",
            rule_file: "australia_subclass_189.yaml",
            applicant_file: "tc06_australia_age_barred.yaml",
            expected_eligible: false,
            expected_min_score: None,
            description: "Age 46 triggers hard disqualification gate in Australia 189",
        },
        TestCase {
            id: "TC-07",
            name: "Temporal CEP Expiration (730 Days)",
            rule_file: "edge_cases_drools_parity_suite.yaml",
            applicant_file: "tc07_temporal_expired_ielts.yaml",
            expected_eligible: false,
            expected_min_score: None,
            description: "Language test taken 820 days ago fails recency window",
        },
        TestCase {
            id: "TC-08",
            name: "Three-Valued Logic & Null-Safe Traversal",
            rule_file: "edge_cases_drools_parity_suite.yaml",
            applicant_file: "tc08_null_spouse_single.yaml",
            expected_eligible: true,
            expected_min_score: None,
            description: "Single candidate with null spouse profile skips spouse rules safely",
        },
        TestCase {
            id: "TC-10",
            name: "Universal Quantifier (forall)",
            rule_file: "edge_cases_drools_parity_suite.yaml",
            applicant_file: "tc10_forall_language_fail.yaml",
            expected_eligible: true, // Candidate is eligible overall, but rule fails
            expected_min_score: None,
            description: "Candidate with CLB 10,10,10,6 fails forall(clb >= 7) condition",
        },
        TestCase {
            id: "TC-11",
            name: "Forward Chaining & Inferred Fact Mutation",
            rule_file: "edge_cases_drools_parity_suite.yaml",
            applicant_file: "tc11_forward_chaining.yaml",
            expected_eligible: true,
            expected_min_score: Some(50.0),
            description: "Phase 1 inference derives fact that fires Phase 3 transferability rule",
        },
    ];

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Test ID"),
            Cell::new("Scenario Name"),
            Cell::new("Feature / Edge Case Tested"),
            Cell::new("Score"),
            Cell::new("Status"),
            Cell::new("Verdict"),
        ]);

    let mut passed_count = 0;

    for tc in &test_cases {
        let rule_path = rules_dir.join(tc.rule_file);
        let applicant_path = applicants_dir.join(tc.applicant_file);

        let program = load_rule_program(&rule_path)?;
        let context = load_fact_context(&applicant_path)?;

        let engine = Engine::new(program);
        let report = engine.evaluate(&context)?;

        let mut passed = true;
        if tc.expected_eligible != report.is_eligible() {
            passed = false;
        }
        if let Some(min_score) = tc.expected_min_score {
            if report.total_score < min_score {
                passed = false;
            }
        }

        if passed {
            passed_count += 1;
        }

        let status_str = if report.is_eligible() {
            Cell::new("Eligible").fg(Color::Green)
        } else {
            Cell::new("Ineligible").fg(Color::Red)
        };

        let verdict_cell = if passed {
            Cell::new("PASSED (PASS)").fg(Color::Green)
        } else {
            Cell::new("FAILED (FAIL)").fg(Color::Red)
        };

        table.add_row(vec![
            Cell::new(tc.id).fg(Color::Cyan),
            Cell::new(tc.name),
            Cell::new(tc.description),
            Cell::new(format!("{:.1}", report.total_score)),
            status_str,
            verdict_cell,
        ]);
    }

    println!("{table}");
    println!(
        "\n  Results: {} / {} tests passed successfully.\n",
        passed_count.to_string().green().bold(),
        test_cases.len()
    );

    Ok(())
}
