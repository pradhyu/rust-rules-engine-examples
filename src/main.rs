use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use rust_rules_engine::{
    AppState, RulesGrpcService, RulesServiceServer, create_rest_router, evaluate_facts,
    json_to_facts, load_knowledge_base_from_path, run_interactive_repl,
};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "rules-engine-cli",
    about = "A high-performance Rules Engine in Rust (powered by KSD-CO/rust-rule-engine) with REST, gRPC, and Interactive REPL support",
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
    /// Evaluate a single applicant profile against GRL rules
    Evaluate {
        /// Path to the GRL rules file or directory
        #[arg(short, long, alias = "rules-dir")]
        rules: PathBuf,

        /// Path to the applicant profile YAML/JSON file
        #[arg(short, long)]
        applicant: PathBuf,

        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },

    /// Batch evaluate and rank all applicants in a directory against GRL rules
    Batch {
        /// Path to the GRL rules file or directory
        #[arg(short, long, alias = "rules-dir")]
        rules: PathBuf,

        /// Directory containing applicant profile YAML/JSON files
        #[arg(short, long)]
        applicants_dir: PathBuf,

        /// Minimum cutoff score filter (e.g. for Express Entry invitation draws)
        #[arg(short, long)]
        cutoff: Option<f64>,
    },

    /// Inspect a Knowledge Base rules file or directory
    Inspect {
        /// Path to the GRL rules file or directory
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
        /// Path to the GRL rules file or directory to load initially
        #[arg(short, long, alias = "rules-dir", default_value = "rules/uk_skilled_worker_points.grl")]
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
            let rules_str = rules.to_string_lossy().to_string();
            let state = AppState::from_path(&rules_str)?;
            let kb_name = state.kb.read().await.name().to_string();
            let rule_count = state.kb.read().await.rule_count();

            let rest_addr: SocketAddr = format!("0.0.0.0:{}", rest_port).parse()?;
            let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;

            println!(
                "\n{}",
                " 🚀 STARTING REAL-TIME RULES ENGINE MICROSERVICE (via KSD-CO/rust-rule-engine) 🚀 "
                    .bold()
                    .on_green()
                    .black()
            );
            println!(
                "  • Loaded Knowledge Base: {} ({} rules)",
                kb_name.cyan().bold(),
                rule_count
            );
            println!(
                "  • REST Endpoint:         {}",
                format!("http://localhost:{}/api/v1/evaluate", rest_port)
                    .yellow()
                    .bold()
            );
            println!(
                "  • gRPC Service:          {}",
                format!("http://localhost:{}", grpc_port).yellow().bold()
            );
            println!(
                "  • Health Check:          {}",
                format!("http://localhost:{}/health", rest_port).dimmed()
            );
            println!(
                "  • Inspection:            {}\n",
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
            let rules_str = rules.to_string_lossy().to_string();
            let (kb, pass_mark) = load_knowledge_base_from_path(&rules_str)?;
            let facts = load_fact_payload(&applicant)?;

            let report = evaluate_facts(&kb, &facts, pass_mark)?;

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
            let rules_str = rules.to_string_lossy().to_string();
            let (kb, pass_mark) = load_knowledge_base_from_path(&rules_str)?;

            let mut results = Vec::new();
            let start = Instant::now();

            for entry in fs::read_dir(&applicants_dir)? {
                let entry = entry?;
                let path = entry.path();
                let is_data_file = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e == "yaml" || e == "yml" || e == "json");
                if is_data_file {
                    if let Ok(facts) = load_fact_payload(&path) {
                        if let Ok(report) = evaluate_facts(&kb, &facts, pass_mark) {
                            let file_name = path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            results.push((file_name, report));
                        }
                    }
                }
            }

            // Sort by total score descending (Ranking)
            results.sort_by(|a, b| {
                b.1.total_score
                    .partial_cmp(&a.1.total_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let elapsed = start.elapsed();
            println!(
                "\n{}",
                format!(" 🏆 BATCH RANKING & SELECTION DRAW: {} ", kb.name())
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
            println!(
                "  Total Candidates Evaluated: {} (in {:.2} ms)",
                results.len(),
                elapsed.as_secs_f64() * 1000.0
            );

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
            let rules_str = rules.to_string_lossy().to_string();
            let (kb, pass_mark) = load_knowledge_base_from_path(&rules_str)?;
            println!(
                "\n{}",
                format!(" 🔍 KNOWLEDGE BASE INSPECTION: {} ", kb.name())
                    .bold()
                    .on_cyan()
                    .black()
            );
            println!("  Rule Count: {}", kb.rule_count().to_string().yellow().bold());
            if let Some(pass) = pass_mark {
                println!(
                    "  Pass Mark Threshold: {}",
                    format!("{:.1} points", pass).yellow().bold()
                );
            }

            let rules = kb.get_rules();
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Salience").fg(Color::Yellow),
                    Cell::new("Rule Name").fg(Color::Cyan),
                    Cell::new("Activation Group").fg(Color::Magenta),
                    Cell::new("Description").fg(Color::White),
                ]);

            for r in rules {
                table.add_row(vec![
                    Cell::new(r.salience.to_string()).fg(Color::Yellow),
                    Cell::new(&r.name).fg(Color::Cyan),
                    Cell::new(r.activation_group.as_deref().unwrap_or("none")).fg(Color::Magenta),
                    Cell::new(r.description.as_deref().unwrap_or("")),
                ]);
            }

            println!("{table}\n");
        }
    }

    Ok(())
}

fn load_fact_payload(path: &Path) -> Result<rust_rule_engine::Facts, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let val: serde_json::Value = if path.extension().and_then(|e| e.to_str()) == Some("json") {
        serde_json::from_str(&content)?
    } else {
        serde_yaml::from_str(&content)?
    };
    Ok(json_to_facts(&val))
}
