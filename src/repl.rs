use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::core::{Engine, FactContext, RuleProgram};

/// Start the Interactive Rules Engine REPL
pub fn run_interactive_repl(
    initial_rules_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history(".rules_repl_history");

    // Load initial rule program if specified, or default to canada_crs
    let default_path = initial_rules_path.unwrap_or_else(|| "rules/canada_crs/".to_string());
    let mut current_rules_path = default_path.clone();
    let mut engine: Option<Engine> = match load_program(&current_rules_path) {
        Ok(prog) => Some(Engine::new(prog)),
        Err(e) => {
            println!(
                "{} Failed to load initial rules '{}': {}",
                "⚠️".yellow(),
                current_rules_path.yellow(),
                e
            );
            None
        }
    };

    println!(
        "\n{}",
        " 🚀 RUST RULES ENGINE INTERACTIVE REPL 🚀 "
            .bold()
            .on_purple()
            .white()
    );
    println!(
        " Type {} to see available commands, {} to exit.",
        ":help".yellow().bold(),
        ":quit".yellow().bold()
    );
    print_active_status(&current_rules_path, &engine);

    loop {
        let prompt_program_name = engine
            .as_ref()
            .map(|e| e.program().id.clone())
            .unwrap_or_else(|| "no-rules".to_string());

        let prompt = format!(
            "{} [{}]> ",
            "rules-engine".magenta().bold(),
            prompt_program_name.cyan()
        );
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(trimmed);

                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let command = parts[0];
                let args = &parts[1..];

                match command {
                    ":help" | "?" | "help" => {
                        print_help();
                    }
                    ":load" => {
                        if args.is_empty() {
                            println!("{} Usage: :load <path-to-file-or-dir>", "⚠️".yellow());
                            continue;
                        }
                        let path = args[0];
                        match load_program(path) {
                            Ok(prog) => {
                                println!(
                                    "{} Successfully loaded ruleset: {} (v{}) with {} rules.",
                                    "✅".green().bold(),
                                    prog.name.cyan(),
                                    prog.version,
                                    prog.rules.len()
                                );
                                current_rules_path = path.to_string();
                                engine = Some(Engine::new(prog));
                            }
                            Err(e) => {
                                println!(
                                    "{} Error loading rules from '{}': {}",
                                    "❌".red().bold(),
                                    path,
                                    e
                                );
                            }
                        }
                    }
                    ":inspect" => {
                        if let Some(eng) = &engine {
                            inspect_engine(eng, &current_rules_path);
                        } else {
                            println!(
                                "{} No ruleset currently loaded. Use ':load <path>' first.",
                                "⚠️".yellow()
                            );
                        }
                    }
                    ":status" => {
                        print_active_status(&current_rules_path, &engine);
                    }
                    ":eval" => {
                        if args.is_empty() {
                            println!(
                                "{} Usage: :eval <path-to-applicant-yaml-or-json>",
                                "⚠️".yellow()
                            );
                            continue;
                        }
                        if let Some(eng) = &engine {
                            let app_path = args[0];
                            eval_file(eng, app_path);
                        } else {
                            println!(
                                "{} No ruleset loaded. Use ':load <path>' first.",
                                "⚠️".yellow()
                            );
                        }
                    }
                    ":eval-json" => {
                        if args.is_empty() {
                            println!("{} Usage: :eval-json <raw-json-fact-string>", "⚠️".yellow());
                            continue;
                        }
                        if let Some(eng) = &engine {
                            let json_str = trimmed.strip_prefix(":eval-json").unwrap_or("").trim();
                            eval_json_str(eng, json_str);
                        } else {
                            println!(
                                "{} No ruleset loaded. Use ':load <path>' first.",
                                "⚠️".yellow()
                            );
                        }
                    }
                    ":whatif" => {
                        if args.is_empty() {
                            println!(
                                "{} Usage: :whatif <path-to-applicant> [cutoff_score]",
                                "⚠️".yellow()
                            );
                            continue;
                        }
                        if let Some(eng) = &engine {
                            let app_path = args[0];
                            let cutoff: f64 =
                                args.get(1).and_then(|s| s.parse().ok()).unwrap_or(485.0);
                            run_whatif(eng, app_path, cutoff);
                        } else {
                            println!(
                                "{} No ruleset loaded. Use ':load <path>' first.",
                                "⚠️".yellow()
                            );
                        }
                    }
                    ":batch" => {
                        if args.is_empty() {
                            println!(
                                "{} Usage: :batch <applicants-directory> [cutoff_score]",
                                "⚠️".yellow()
                            );
                            continue;
                        }
                        if let Some(eng) = &engine {
                            let dir_path = args[0];
                            let cutoff: Option<f64> = args.get(1).and_then(|s| s.parse().ok());
                            run_batch(eng, dir_path, cutoff);
                        } else {
                            println!(
                                "{} No ruleset loaded. Use ':load <path>' first.",
                                "⚠️".yellow()
                            );
                        }
                    }
                    ":stats" | ":benchmark" => {
                        if let Some(eng) = &engine {
                            let iterations: usize =
                                args.first().and_then(|s| s.parse().ok()).unwrap_or(10_000);
                            run_benchmark(eng, iterations);
                        } else {
                            println!(
                                "{} No ruleset loaded. Use ':load <path>' first.",
                                "⚠️".yellow()
                            );
                        }
                    }
                    ":clear" | "clear" => {
                        print!("\x1B[2J\x1B[1;1H");
                    }
                    ":quit" | ":exit" | "quit" | "exit" => {
                        println!("{} Exiting REPL. Goodbye!", "👋".cyan());
                        break;
                    }
                    cmd if cmd.starts_with(':') => {
                        println!(
                            "{} Unknown command '{}'. Type ':help' for assistance.",
                            "❓".red(),
                            cmd
                        );
                    }
                    _ => {
                        // Attempt to evaluate as inline fact path or JSON
                        if let Some(eng) = &engine {
                            if Path::new(command).exists() {
                                eval_file(eng, command);
                            } else {
                                println!(
                                    "{} Unrecognized input. Type ':help' for command syntax or ':eval <file>'.",
                                    "❓".yellow()
                                );
                            }
                        } else {
                            println!(
                                "{} No rules loaded. Type ':load <path>' to get started.",
                                "⚠️".yellow()
                            );
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("\n{} Exiting REPL.", "👋".cyan());
                break;
            }
            Err(err) => {
                println!("{} Readline error: {}", "❌".red(), err);
                break;
            }
        }
    }

    let _ = rl.save_history(".rules_repl_history");
    Ok(())
}

fn load_program(path_str: &str) -> Result<RuleProgram, Box<dyn std::error::Error>> {
    let p = Path::new(path_str);
    if p.is_dir() {
        RuleProgram::from_directory(p)
    } else {
        RuleProgram::from_file(p)
    }
}

fn print_active_status(path: &str, engine: &Option<Engine>) {
    if let Some(e) = engine {
        let p = e.program();
        println!(
            " Active Ruleset: {} (v{}) [{} rules]",
            p.name.cyan().bold(),
            p.version,
            p.rules.len()
        );
        println!(" Source Path:    {}\n", path.yellow());
    } else {
        println!(" {} No active ruleset loaded.\n", "⚠️".yellow());
    }
}

fn print_help() {
    println!(
        "\n{}",
        "📋 INTERACTIVE REPL COMMAND PALETTE".bold().underline()
    );
    println!(
        "  {:<28} Load a rule file or modular rule directory",
        ":load <path|dir>".cyan().bold()
    );
    println!(
        "  {:<28} Inspect active ruleset metadata, caps, and rule phases",
        ":inspect".cyan().bold()
    );
    println!(
        "  {:<28} Display active ruleset name and source path",
        ":status".cyan().bold()
    );
    println!(
        "  {:<28} Evaluate an applicant fact file (YAML or JSON)",
        ":eval <path>".cyan().bold()
    );
    println!(
        "  {:<28} Evaluate inline JSON fact payload",
        ":eval-json <raw_json>".cyan().bold()
    );
    println!(
        "  {:<28} Run real-time What-If simulation and pathway recommendations",
        ":whatif <path> [cutoff]".cyan().bold()
    );
    println!(
        "  {:<28} Batch evaluate all applicant files in directory & rank",
        ":batch <dir> [cutoff]".cyan().bold()
    );
    println!(
        "  {:<28} Benchmark evaluation throughput & latency percentiles",
        ":stats [iterations]".cyan().bold()
    );
    println!("  {:<28} Clear terminal screen", ":clear".cyan().bold());
    println!("  {:<28} Display this help menu", ":help, ?".cyan().bold());
    println!(
        "  {:<28} Exit the interactive REPL\n",
        ":quit, :exit".cyan().bold()
    );
}

fn inspect_engine(engine: &Engine, path: &str) {
    let p = engine.program();
    println!(
        "\n{}",
        format!(" 🔍 ACTIVE RULESET: {} (v{}) ", p.name, p.version)
            .bold()
            .on_cyan()
            .black()
    );
    println!("Source Path: {}", path.yellow());
    if let Some(desc) = &p.description {
        println!("Description: {}", desc.dimmed());
    }
    if let Some(pass) = p.pass_mark_threshold {
        println!(
            "Pass Threshold: {}",
            format!("{:.1} points", pass).yellow().bold()
        );
    }
    if let Some(total_cap) = p.total_points_cap {
        println!(
            "Total Score Cap: {}",
            format!("{:.1} points", total_cap).yellow().bold()
        );
    }

    println!("\n{}", "📊 Category Point Caps:".bold().underline());
    for cat in p.categories.values() {
        let cap_str = cat
            .max_points
            .map(|m| format!("{:.1} pts", m))
            .unwrap_or_else(|| "No Cap".to_string());
        println!(
            "  • {:<30} -> Max Cap: {}",
            cat.display_name.cyan(),
            cap_str.yellow()
        );
    }

    println!("\n{}", "📜 Defined Rules by Phase:".bold().underline());
    let mut rules_by_phase: std::collections::BTreeMap<String, Vec<&crate::core::Rule>> =
        std::collections::BTreeMap::new();
    for r in &p.rules {
        rules_by_phase.entry(r.phase.clone()).or_default().push(r);
    }
    for (phase, rules) in rules_by_phase {
        println!(
            "  [{}] ({} rules)",
            phase.to_uppercase().magenta().bold(),
            rules.len()
        );
        for r in rules {
            let prio = format!("(salience: {})", r.priority).dimmed();
            let gate = if r.is_eligibility_gate {
                " [GATE]".red().bold()
            } else {
                "".normal()
            };
            println!("    - {:<36} {}{}", r.name.white(), prio, gate);
        }
    }
    println!();
}

fn eval_file(engine: &Engine, path_str: &str) {
    let p = Path::new(path_str);
    if !p.exists() {
        println!("{} Applicant file not found: {}", "❌".red(), path_str);
        return;
    }

    let content = match fs::read_to_string(p) {
        Ok(c) => c,
        Err(e) => {
            println!("{} Read error: {}", "❌".red(), e);
            return;
        }
    };

    let val: serde_json::Value = if p.extension().and_then(|e| e.to_str()) == Some("json") {
        match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                println!("{} JSON parse error: {}", "❌".red(), e);
                return;
            }
        }
    } else {
        match serde_yaml::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                println!("{} YAML parse error: {}", "❌".red(), e);
                return;
            }
        }
    };

    let start = Instant::now();
    let ctx = FactContext::from_value(val);
    match engine.evaluate(&ctx) {
        Ok(report) => {
            let latency = start.elapsed();
            println!(
                "\n⚡ Evaluated in: {}",
                format!("{:.2}µs", latency.as_secs_f64() * 1_000_000.0)
                    .green()
                    .bold()
            );
            report.print_audit_table();
        }
        Err(e) => {
            println!("{} Evaluation error: {}", "❌".red(), e);
        }
    }
}

fn eval_json_str(engine: &Engine, json_str: &str) {
    let val: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            println!("{} Invalid JSON fact input: {}", "❌".red(), e);
            return;
        }
    };

    let start = Instant::now();
    let ctx = FactContext::from_value(val);
    match engine.evaluate(&ctx) {
        Ok(report) => {
            let latency = start.elapsed();
            println!(
                "\n⚡ Evaluated in: {}",
                format!("{:.2}µs", latency.as_secs_f64() * 1_000_000.0)
                    .green()
                    .bold()
            );
            report.print_audit_table();
        }
        Err(e) => {
            println!("{} Evaluation error: {}", "❌".red(), e);
        }
    }
}

fn run_whatif(engine: &Engine, app_path: &str, cutoff: f64) {
    let p = Path::new(app_path);
    if !p.exists() {
        println!("{} File not found: {}", "❌".red(), app_path);
        return;
    }
    let content = match fs::read_to_string(p) {
        Ok(c) => c,
        Err(e) => {
            println!("{} Error reading file: {}", "❌".red(), e);
            return;
        }
    };

    let base_val: serde_json::Value = if p.extension().and_then(|e| e.to_str()) == Some("json") {
        serde_json::from_str(&content).unwrap_or(serde_json::Value::Null)
    } else {
        serde_yaml::from_str(&content).unwrap_or(serde_json::Value::Null)
    };

    let base_ctx = FactContext::from_value(base_val.clone());
    let base_report = match engine.evaluate(&base_ctx) {
        Ok(r) => r,
        Err(e) => {
            println!("{} Base evaluation error: {}", "❌".red(), e);
            return;
        }
    };

    let current = base_report.total_score;
    println!(
        "\n{}",
        " 🍁 REAL-TIME WHAT-IF SIMULATION ADVISOR 🍁 "
            .bold()
            .on_cyan()
            .black()
    );
    println!(
        " Candidate ID:   {}",
        base_report
            .applicant_id
            .unwrap_or_else(|| app_path.to_string())
            .yellow()
            .bold()
    );
    println!(" Current Score:  {:.1} points", current.to_string().cyan());
    println!(" Target Cutoff:  {:.1} points", cutoff.to_string().yellow());

    if current >= cutoff {
        println!(
            " Status:         {}\n",
            "QUALIFIED FOR INVITATION ROUND".green().bold()
        );
    } else {
        println!(
            " Status:         {} (Need {:.1} more points)\n",
            "BELOW DRAW CUTOFF".red().bold(),
            cutoff - current
        );
    }

    println!("{}", "💡 Actionable Pathway Options:".bold().underline());

    // Option 1: PNP
    let mut pnp = base_val.clone();
    pnp["applicant"]["additional_factors"]["has_provincial_nomination"] =
        serde_json::Value::Bool(true);
    if let Ok(rep) = engine.evaluate(&FactContext::from_value(pnp)) {
        let gain = rep.total_score - current;
        if gain > 0.0 {
            println!(
                "  1. {} [+{:.1} pts → Projected: {:.1}]",
                "Secure Provincial Nomination (PNP)".cyan().bold(),
                gain,
                rep.total_score
            );
            println!(
                "     {}",
                "Guarantees Invitation to Apply (ITA) in next targeted draw.".dimmed()
            );
        }
    }

    // Option 2: Language CLB 9+
    let mut lang = base_val.clone();
    lang["applicant"]["language"]["first_official"]["clb_reading"] = serde_json::json!(9);
    lang["applicant"]["language"]["first_official"]["clb_writing"] = serde_json::json!(9);
    lang["applicant"]["language"]["first_official"]["clb_listening"] = serde_json::json!(9);
    lang["applicant"]["language"]["first_official"]["clb_speaking"] = serde_json::json!(9);
    if let Ok(rep) = engine.evaluate(&FactContext::from_value(lang)) {
        let gain = rep.total_score - current;
        if gain > 0.0 {
            println!(
                "  2. {} [+{:.1} pts → Projected: {:.1}]",
                "Retake Language Exam to reach CLB 9+".cyan().bold(),
                gain,
                rep.total_score
            );
            println!(
                "     {}",
                "Unlocks maximum Skill Transferability multipliers.".dimmed()
            );
        }
    }

    // Option 3: Additional Canadian Work Year
    let mut work = base_val.clone();
    let curr_work = work["applicant"]["work_experience"]["domestic_years"]
        .as_i64()
        .unwrap_or(0);
    work["applicant"]["work_experience"]["domestic_years"] = serde_json::json!(curr_work + 1);
    if let Ok(rep) = engine.evaluate(&FactContext::from_value(work)) {
        let gain = rep.total_score - current;
        if gain > 0.0 {
            println!(
                "  3. {} [+{:.1} pts → Projected: {:.1}]",
                format!("Complete {} year(s) Canadian Domestic Work", curr_work + 1)
                    .cyan()
                    .bold(),
                gain,
                rep.total_score
            );
            println!(
                "     {}",
                "Increases Core Human Capital domestic work points.".dimmed()
            );
        }
    }
    println!();
}

fn run_batch(engine: &Engine, dir_path: &str, cutoff: Option<f64>) {
    let p = Path::new(dir_path);
    if !p.is_dir() {
        println!("{} Path is not a directory: {}", "❌".red(), dir_path);
        return;
    }

    let mut results = Vec::new();
    let entries = match fs::read_dir(p) {
        Ok(e) => e,
        Err(err) => {
            println!("{} Read dir error: {}", "❌".red(), err);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let is_data = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e == "yaml" || e == "json");
        if is_data {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let val: serde_json::Value =
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    serde_json::from_str(&content).unwrap_or(serde_json::Value::Null)
                } else {
                    serde_yaml::from_str(&content).unwrap_or(serde_json::Value::Null)
                };
            if let Ok(rep) = engine.evaluate(&FactContext::from_value(val)) {
                let id = rep.applicant_id.clone().unwrap_or_else(|| {
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });
                results.push((id, rep));
            }
        }
    }

    results.sort_by(|a, b| {
        b.1.total_score
            .partial_cmp(&a.1.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    println!(
        "\n{}",
        format!(" 🏆 BATCH RANKING: {} ", engine.program().name)
            .bold()
            .on_purple()
            .white()
    );
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Rank"),
            Cell::new("Candidate ID"),
            Cell::new("Total Score"),
            Cell::new("Status"),
            Cell::new("Draw Result"),
        ]);

    for (idx, (id, report)) in results.iter().enumerate() {
        let rank = (idx + 1).to_string();
        let (status_str, draw_res) = if !report.is_eligible() {
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
            Cell::new(id),
            Cell::new(format!("{:.1}", report.total_score)).fg(Color::Cyan),
            status_str,
            draw_res,
        ]);
    }
    println!("{table}\n");
}

fn run_benchmark(engine: &Engine, iterations: usize) {
    // Generate sample candidate
    let fact_val = serde_json::json!({
        "applicant": {
            "id": "BENCH-001",
            "age": 29,
            "marital_status": "single",
            "education": { "highest_degree": "master" },
            "language": {
                "first_official": { "clb_reading": 9, "clb_writing": 9, "clb_listening": 9, "clb_speaking": 9 }
            },
            "work_experience": { "domestic_years": 2, "foreign_years": 3 },
            "additional_factors": { "has_provincial_nomination": true }
        }
    });

    let ctx = FactContext::from_value(fact_val);
    let mut durations = Vec::with_capacity(iterations);

    println!(
        " Running {} evaluations on {}...",
        iterations,
        engine.program().name.cyan()
    );
    let total_start = Instant::now();

    for _ in 0..iterations {
        let t0 = Instant::now();
        let _ = engine.evaluate(&ctx);
        durations.push(t0.elapsed().as_secs_f64() * 1_000_000.0);
    }

    let total_elapsed = total_start.elapsed();
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p50 = durations[iterations / 2];
    let p95 = durations[(iterations as f64 * 0.95) as usize];
    let p99 = durations[(iterations as f64 * 0.99) as usize];
    let throughput = iterations as f64 / total_elapsed.as_secs_f64();

    println!(
        "\n{}",
        " ⚡ REAL-TIME BENCHMARK METRICS ⚡ "
            .bold()
            .on_green()
            .black()
    );
    println!("  • Iterations:        {}", iterations);
    println!("  • Total Time:        {:.2?}", total_elapsed);
    println!(
        "  • Throughput:        {} evals/sec",
        format!("{:.0}", throughput).yellow().bold()
    );
    println!("  • Latency Median (p50): {:.2} µs", p50);
    println!("  • Latency 95th (p95):   {:.2} µs", p95);
    println!("  • Latency 99th (p99):   {:.2} µs\n", p99);
}
