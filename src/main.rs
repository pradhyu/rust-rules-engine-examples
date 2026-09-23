use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use rust_rules_engine::{
    AppState, Engine, FactContext, RuleProgram, RulesGrpcService, RulesServiceServer,
    create_rest_router, run_interactive_repl,
};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "rules-engine-cli",
    about = "A high-performance, deterministic Rules Engine in Rust with REST, gRPC, and Interactive REPL support",
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

    /// Start the interactive terminal REPL for real-time rule evaluation & simulations
    Repl {
        /// Optional initial rule file or directory to load
        #[arg(short, long, alias = "rules-dir")]
        rules: Option<PathBuf>,
    },

    /// Run real-time high-throughput REST (HTTP/JSON) and gRPC (HTTP/2) microservices
    Serve {
        /// Path to the rules folder or file to load initially
        #[arg(short, long, alias = "rules-dir", default_value = "rules/canada_crs/")]
        rules: PathBuf,

        /// HTTP REST API port
        #[arg(long, default_value_t = 8080)]
        rest_port: u16,

        /// gRPC service port
        #[arg(long, default_value_t = 50051)]
        grpc_port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Repl { rules } => {
            let rules_str = rules.map(|p| p.to_string_lossy().to_string());
            run_interactive_repl(rules_str)?;
        }

        Commands::Serve {
            rules,
            rest_port,
            grpc_port,
        } => {
            let program = load_rule_program(&rules)?;
            let program_name = program.name.clone();
            let rule_count = program.rules.len();
            let state = AppState::new(program);

            let rest_addr: SocketAddr = format!("0.0.0.0:{}", rest_port).parse()?;
            let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;

            println!(
                "\n{}",
                " 🚀 STARTING REAL-TIME RULES ENGINE MICROSERVICE 🚀 "
                    .bold()
                    .on_green()
                    .black()
            );
            println!(
                "  • Loaded Ruleset:  {} ({} rules)",
                program_name.cyan().bold(),
                rule_count
            );
            println!(
                "  • REST Endpoint:   {}",
                format!("http://localhost:{}/api/v1/evaluate", rest_port)
                    .yellow()
                    .bold()
            );
            println!(
                "  • gRPC Service:    {}",
                format!("http://localhost:{}", grpc_port).yellow().bold()
            );
            println!(
                "  • Health Check:    {}",
                format!("http://localhost:{}/health", rest_port).dimmed()
            );
            println!(
                "  • Inspection:      {}\n",
                format!("http://localhost:{}/api/v1/inspect", rest_port).dimmed()
            );

            let rest_router = create_rest_router(state.clone());
            let grpc_service = RulesGrpcService::new(state);

            let rest_handle = tokio::spawn(async move {
                let listener = tokio::net::TcpListener::bind(rest_addr)
                    .await
                    .expect("Failed to bind REST port");
                axum::serve(listener, rest_router)
                    .await
                    .expect("REST server error");
            });

            let grpc_handle = tokio::spawn(async move {
                tonic::transport::Server::builder()
                    .add_service(RulesServiceServer::new(grpc_service))
                    .serve(grpc_addr)
                    .await
                    .expect("gRPC server error");
            });

            tokio::select! {
                _ = rest_handle => {},
                _ = grpc_handle => {},
            }
        }

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
                let is_data_file = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e == "yaml" || e == "json");
                if is_data_file
                    && let Ok(ctx) = load_fact_context(&path)
                    && let Ok(report) = engine.evaluate(&ctx)
                {
                    let file_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    results.push((file_name, report));
                }
            }

            // Sort by total score descending (Ranking)
            results.sort_by(|a, b| {
                b.1.total_score
                    .partial_cmp(&a.1.total_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            println!(
                "\n{}",
                format!(
                    " 🏆 BATCH RANKING & SELECTION DRAW: {} ",
                    engine.program().name
                )
                .bold()
                .on_purple()
                .white()
            );
            if let Some(cut) = cutoff {
                println!(
                    "  Cutoff Score Threshold: {}",
                    format!("{:.1} points", cut).yellow().bold()
                );
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
                    (
                        Cell::new("Ineligible").fg(Color::Red),
                        Cell::new("Disqualified").fg(Color::Red),
                    )
                } else if let Some(cut) = cutoff {
                    if report.total_score >= cut {
                        (
                            Cell::new("Eligible").fg(Color::Green),
                            Cell::new("SELECTED (ITA)").fg(Color::Green),
                        )
                    } else {
                        (
                            Cell::new("Eligible").fg(Color::Green),
                            Cell::new("Below Cutoff").fg(Color::Yellow),
                        )
                    }
                } else {
                    (
                        Cell::new("Eligible").fg(Color::Green),
                        Cell::new("Ranked").fg(Color::Cyan),
                    )
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
            println!(
                "\n{}",
                format!(
                    " 🔍 PROGRAM INSPECTION: {} (v{}) ",
                    program.name, program.version
                )
                .bold()
                .on_cyan()
                .black()
            );
            if let Some(desc) = &program.description {
                println!("Description: {}", desc.dimmed());
            }
            if let Some(pass) = program.pass_mark_threshold {
                println!(
                    "Pass Mark Threshold: {}",
                    format!("{:.1} points", pass).yellow().bold()
                );
            }
            if let Some(total_cap) = program.total_points_cap {
                println!(
                    "Overall Total Cap: {}",
                    format!("{:.1} points", total_cap).yellow().bold()
                );
            }

            println!("\n{}", "📊 Category Budgets & Sub-Caps:".bold().underline());
            for cat in program.categories.values() {
                let cap_str = cat
                    .max_points
                    .map(|p| format!("{:.1} pts", p))
                    .unwrap_or_else(|| "No Cap".to_string());
                println!(
                    "  • {:<25} -> Max: {}",
                    cat.display_name.cyan(),
                    cap_str.yellow()
                );
            }

            println!("\n{}", "📜 Defined Rules by Phase:".bold().underline());
            let mut rules_by_phase: std::collections::BTreeMap<
                String,
                Vec<&rust_rules_engine::Rule>,
            > = std::collections::BTreeMap::new();
            for r in &program.rules {
                rules_by_phase.entry(r.phase.clone()).or_default().push(r);
            }

            for (phase, rules) in rules_by_phase {
                println!(
                    "\n  [{}] ({} rules)",
                    phase.to_uppercase().magenta().bold(),
                    rules.len()
                );
                for r in rules {
                    let gate_tag = if r.is_eligibility_gate {
                        "[GATE]".red()
                    } else {
                        "".normal()
                    };
                    let group_tag = r
                        .activation_group
                        .as_ref()
                        .map(|g| format!("[XOR: {}]", g).yellow())
                        .unwrap_or_default();
                    println!(
                        "    • {:<35} (Priority: {:>4}) {} {}",
                        r.name, r.priority, gate_tag, group_tag
                    );
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
