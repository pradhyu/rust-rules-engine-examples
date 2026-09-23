use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use rust_rule_engine::KnowledgeBase;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::evaluator::{evaluate_facts, json_to_facts, load_knowledge_base_from_path};

pub struct LoadedRuleset {
    pub kb: KnowledgeBase,
    pub pass_mark: Option<f64>,
    pub path: String,
}

/// Start the Interactive Rules Engine REPL
pub fn run_interactive_repl(
    initial_rules_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history(".rules_repl_history");

    let default_path = initial_rules_path.unwrap_or_else(|| "rules/uk_skilled_worker_points.grl".to_string());
    let mut current_rules_path = default_path.clone();
    let mut ruleset: Option<LoadedRuleset> = match load_knowledge_base_from_path(&current_rules_path) {
        Ok((kb, pass_mark)) => Some(LoadedRuleset {
            kb,
            pass_mark,
            path: current_rules_path.clone(),
        }),
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
        " 🚀 RUST RULES ENGINE INTERACTIVE REPL (Powered by KSD-CO/rust-rule-engine) 🚀 "
            .bold()
            .on_purple()
            .white()
    );
    println!(
        " Type {} to see available commands, {} to exit.",
        ":help".yellow().bold(),
        ":quit".yellow().bold()
    );
    print_active_status(&ruleset);

    loop {
        let prompt_name = ruleset
            .as_ref()
            .map(|r| r.kb.name().to_string())
            .unwrap_or_else(|| "no-rules".to_string());

        let prompt = format!(
            "{} [{}]> ",
            "rust-rule-engine".magenta().bold(),
            prompt_name.cyan()
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
                        match load_knowledge_base_from_path(path) {
                            Ok((kb, pass_mark)) => {
                                println!(
                                    "{} Successfully loaded ruleset: {} with {} rules.",
                                    "✅".green().bold(),
                                    kb.name().cyan(),
                                    kb.rule_count()
                                );
                                current_rules_path = path.to_string();
                                ruleset = Some(LoadedRuleset {
                                    kb,
                                    pass_mark,
                                    path: current_rules_path.clone(),
                                });
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
                        if let Some(r) = &ruleset {
                            inspect_ruleset(r);
                        } else {
                            println!(
                                "{} No ruleset currently loaded. Use ':load <path>' first.",
                                "⚠️".yellow()
                            );
                        }
                    }
                    ":status" => {
                        print_active_status(&ruleset);
                    }
                    ":eval" => {
                        if args.is_empty() {
                            println!(
                                "{} Usage: :eval <path-to-applicant-yaml-or-json>",
                                "⚠️".yellow()
                            );
                            continue;
                        }
                        if let Some(r) = &ruleset {
                            let app_path = args[0];
                            eval_file(r, app_path);
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
                        if let Some(r) = &ruleset {
                            let json_str = trimmed.strip_prefix(":eval-json").unwrap_or("").trim();
                            eval_json_str(r, json_str);
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
                        if let Some(r) = &ruleset {
                            let app_path = args[0];
                            let cutoff: f64 =
                                args.get(1).and_then(|s| s.parse().ok()).unwrap_or(70.0);
                            run_whatif(r, app_path, cutoff);
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
                        if let Some(r) = &ruleset {
                            let dir_path = args[0];
                            let cutoff: Option<f64> = args.get(1).and_then(|s| s.parse().ok());
                            run_batch(r, dir_path, cutoff);
                        } else {
                            println!(
                                "{} No ruleset loaded. Use ':load <path>' first.",
                                "⚠️".yellow()
                            );
                        }
                    }
                    ":stats" | ":benchmark" => {
                        if let Some(r) = &ruleset {
                            let iterations: usize =
                                args.first().and_then(|s| s.parse().ok()).unwrap_or(1000);
                            run_benchmark(r, iterations);
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
                        if let Some(r) = &ruleset {
                            if Path::new(command).exists() {
                                eval_file(r, command);
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
                println!("Exiting...");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(".rules_repl_history");
    Ok(())
}

fn print_active_status(ruleset: &Option<LoadedRuleset>) {
    println!("{}", "─".repeat(70).dimmed());
    if let Some(r) = ruleset {
        println!(
            "  {} Loaded Knowledge Base: {} ({} rules)",
            "🟢".green(),
            r.kb.name().cyan().bold(),
            r.kb.rule_count().to_string().yellow()
        );
        println!("     Path: {}", r.path.dimmed());
        if let Some(m) = r.pass_mark {
            println!("     Pass Mark Threshold: {} points", m.to_string().yellow());
        }
    } else {
        println!("  {} No Knowledge Base loaded.", "🔴".red());
    }
    println!("{}\n", "─".repeat(70).dimmed());
}

fn print_help() {
    println!("\n{}", "📖 AVAILABLE REPL COMMANDS:".bold().underline());
    println!(
        "  {} <path>           Load a GRL rules file or directory into the Knowledge Base",
        ":load".cyan().bold()
    );
    println!(
        "  {}                    Display active Knowledge Base rules, salience & metadata",
        ":inspect".cyan().bold()
    );
    println!(
        "  {}                     Print current ruleset loading status",
        ":status".cyan().bold()
    );
    println!(
        "  {} <path>            Evaluate an applicant YAML/JSON file and print audit trace",
        ":eval".cyan().bold()
    );
    println!(
        "  {} <raw-json>       Evaluate raw inline JSON facts string",
        ":eval-json".cyan().bold()
    );
    println!(
        "  {} <file> [cutoff]  Run counterfactual What-If simulations to reach target cutoff",
        ":whatif".cyan().bold()
    );
    println!(
        "  {} <dir> [cutoff]   Batch evaluate and rank all applicant files in directory",
        ":batch".cyan().bold()
    );
    println!(
        "  {} [iterations]  Run high-throughput microsecond evaluation benchmark",
        ":benchmark".cyan().bold()
    );
    println!(
        "  {}                    Clear the terminal screen",
        ":clear".cyan().bold()
    );
    println!(
        "  {}                     Exit the REPL session",
        ":quit".cyan().bold()
    );
    println!();
}

fn inspect_ruleset(ruleset: &LoadedRuleset) {
    println!(
        "\n{}",
        format!(" 🔍 INSPECTING KNOWLEDGE BASE: {} ", ruleset.kb.name())
            .bold()
            .on_cyan()
            .black()
    );
    println!("  Rule Count: {}", ruleset.kb.rule_count().to_string().yellow().bold());
    if let Some(threshold) = ruleset.pass_mark {
        println!("  Pass Mark:  {} points", threshold.to_string().green().bold());
    }

    let rules = ruleset.kb.get_rules();
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Priority (Salience)").fg(Color::Yellow),
            Cell::new("Rule Name").fg(Color::Cyan),
            Cell::new("Activation Group").fg(Color::Magenta),
            Cell::new("Description").fg(Color::White),
        ]);

    for r in &rules {
        table.add_row(vec![
            Cell::new(r.salience.to_string()).fg(Color::Yellow),
            Cell::new(&r.name).fg(Color::Cyan),
            Cell::new(r.activation_group.as_deref().unwrap_or("none")).fg(Color::Magenta),
            Cell::new(r.description.as_deref().unwrap_or("")),
        ]);
    }
    println!("{table}\n");
}

fn eval_file(ruleset: &LoadedRuleset, file_path: &str) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            println!("{} Failed to read '{}': {}", "❌".red(), file_path, e);
            return;
        }
    };

    let val: serde_json::Value = if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
        match serde_yaml::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                println!("{} Failed to parse YAML: {}", "❌".red(), e);
                return;
            }
        }
    } else {
        match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                println!("{} Failed to parse JSON: {}", "❌".red(), e);
                return;
            }
        }
    };

    let facts = json_to_facts(&val);
    let start = Instant::now();
    match evaluate_facts(&ruleset.kb, &facts, ruleset.pass_mark) {
        Ok(report) => {
            let elapsed = start.elapsed();
            report.print_audit_table();
            println!(
                " ⏱️ Evaluation completed in {:.2} µs\n",
                elapsed.as_secs_f64() * 1_000_000.0
            );
        }
        Err(e) => {
            println!("{} Evaluation error: {}", "❌".red(), e);
        }
    }
}

fn eval_json_str(ruleset: &LoadedRuleset, json_str: &str) {
    let val: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            println!("{} Invalid JSON fact string: {}", "❌".red(), e);
            return;
        }
    };

    let facts = json_to_facts(&val);
    let start = Instant::now();
    match evaluate_facts(&ruleset.kb, &facts, ruleset.pass_mark) {
        Ok(report) => {
            let elapsed = start.elapsed();
            report.print_audit_table();
            println!(
                " ⏱️ Evaluation completed in {:.2} µs\n",
                elapsed.as_secs_f64() * 1_000_000.0
            );
        }
        Err(e) => {
            println!("{} Evaluation error: {}", "❌".red(), e);
        }
    }
}

fn run_whatif(ruleset: &LoadedRuleset, file_path: &str, cutoff: f64) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            println!("{} Failed to read '{}': {}", "❌".red(), file_path, e);
            return;
        }
    };

    let base_val: serde_json::Value = if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
        match serde_yaml::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                println!("{} Failed to parse YAML: {}", "❌".red(), e);
                return;
            }
        }
    } else {
        match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                println!("{} Failed to parse JSON: {}", "❌".red(), e);
                return;
            }
        }
    };

    let base_facts = json_to_facts(&base_val);
    let base_rep = match evaluate_facts(&ruleset.kb, &base_facts, ruleset.pass_mark) {
        Ok(r) => r,
        Err(e) => {
            println!("{} Baseline error: {}", "❌".red(), e);
            return;
        }
    };

    let current = base_rep.total_score;
    println!("\n{}", " 🔮 WHAT-IF PATHWAY SIMULATION ADVISOR 🔮 ".bold().on_blue().white());
    println!(" Applicant File: {}", file_path.cyan());
    println!(" Current Score:  {:.1} points", current.to_string().cyan());
    println!(" Target Cutoff:  {:.1} points", cutoff.to_string().yellow());
    let gap = (cutoff - current).max(0.0);
    println!(" Points Gap:     {:.1} points\n", gap.to_string().magenta().bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Scenario").fg(Color::Cyan),
            Cell::new("Gain").fg(Color::Green),
            Cell::new("Projected Score").fg(Color::Yellow),
            Cell::new("Eligible?").fg(Color::White),
        ]);

    // Scenario 1: Enhanced Provincial Nomination
    let mut pnp_val = base_val.clone();
    pnp_val["applicant"]["additional_factors"]["has_provincial_nomination"] = serde_json::Value::Bool(true);
    let pnp_facts = json_to_facts(&pnp_val);
    if let Ok(rep) = evaluate_facts(&ruleset.kb, &pnp_facts, ruleset.pass_mark) {
        let gain = rep.total_score - current;
        let meets = rep.total_score >= cutoff;
        table.add_row(vec![
            Cell::new("Provincial Nomination (+600 bonus)"),
            Cell::new(format!("+{:.1}", gain)).fg(Color::Green),
            Cell::new(format!("{:.1}", rep.total_score)).fg(Color::Yellow),
            Cell::new(if meets { "YES (ITA Guaranteed)" } else { "No" }),
        ]);
    }

    // Scenario 2: Language Mastery (CLB 9 across all abilities)
    let mut lang_val = base_val.clone();
    lang_val["applicant"]["language"]["first_official"]["clb_reading"] = serde_json::json!(9);
    lang_val["applicant"]["language"]["first_official"]["clb_writing"] = serde_json::json!(9);
    lang_val["applicant"]["language"]["first_official"]["clb_listening"] = serde_json::json!(9);
    lang_val["applicant"]["language"]["first_official"]["clb_speaking"] = serde_json::json!(9);
    let lang_facts = json_to_facts(&lang_val);
    if let Ok(rep) = evaluate_facts(&ruleset.kb, &lang_facts, ruleset.pass_mark) {
        let gain = rep.total_score - current;
        let meets = rep.total_score >= cutoff;
        table.add_row(vec![
            Cell::new("Retake Language Test -> CLB 9+ all abilities"),
            Cell::new(format!("+{:.1}", gain)).fg(Color::Green),
            Cell::new(format!("{:.1}", rep.total_score)).fg(Color::Yellow),
            Cell::new(if meets { "YES" } else { "No" }),
        ]);
    }

    // Scenario 3: +1 Year Canadian Experience
    let mut work_val = base_val.clone();
    let curr_work = work_val["applicant"]["work_experience"]["domestic_years"].as_i64().unwrap_or(0);
    work_val["applicant"]["work_experience"]["domestic_years"] = serde_json::json!(curr_work + 1);
    let work_facts = json_to_facts(&work_val);
    if let Ok(rep) = evaluate_facts(&ruleset.kb, &work_facts, ruleset.pass_mark) {
        let gain = rep.total_score - current;
        let meets = rep.total_score >= cutoff;
        table.add_row(vec![
            Cell::new(format!("+1 Year Domestic Work Experience ({} -> {})", curr_work, curr_work + 1)),
            Cell::new(format!("+{:.1}", gain)).fg(Color::Green),
            Cell::new(format!("{:.1}", rep.total_score)).fg(Color::Yellow),
            Cell::new(if meets { "YES" } else { "No" }),
        ]);
    }

    println!("{table}\n");
}

fn run_batch(ruleset: &LoadedRuleset, dir_path: &str, cutoff: Option<f64>) {
    let dir = Path::new(dir_path);
    if !dir.is_dir() {
        println!("{} '{}' is not a directory.", "❌".red(), dir_path);
        return;
    }

    let mut entries = match fs::read_dir(dir) {
        Ok(e) => e.filter_map(|r| r.ok().map(|e| e.path())).collect::<Vec<_>>(),
        Err(e) => {
            println!("{} Failed to read directory: {}", "❌".red(), e);
            return;
        }
    };
    entries.sort();

    let mut results = Vec::new();
    let start = Instant::now();

    for path in &entries {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "json" && ext != "yaml" && ext != "yml" {
            continue;
        }

        if let Ok(content) = fs::read_to_string(path) {
            let val_res: Result<serde_json::Value, _> = if ext == "yaml" || ext == "yml" {
                serde_yaml::from_str(&content).map_err(|e| e.to_string())
            } else {
                serde_json::from_str(&content).map_err(|e| e.to_string())
            };

            if let Ok(val) = val_res {
                let facts = json_to_facts(&val);
                if let Ok(rep) = evaluate_facts(&ruleset.kb, &facts, ruleset.pass_mark) {
                    let id = rep.applicant_id.clone().unwrap_or_else(|| {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown")
                            .to_string()
                    });
                    results.push((id, rep));
                }
            }
        }
    }

    let elapsed = start.elapsed();
    results.sort_by(|a, b| b.1.total_score.partial_cmp(&a.1.total_score).unwrap_or(std::cmp::Ordering::Equal));

    println!(
        "\n{}",
        format!(" 📊 BATCH EVALUATION RESULTS ({}) ", dir_path)
            .bold()
            .on_cyan()
            .black()
    );
    println!(" Evaluated {} applicants in {:.2} ms", results.len(), elapsed.as_secs_f64() * 1000.0);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Rank").fg(Color::Yellow),
            Cell::new("Candidate ID").fg(Color::Cyan),
            Cell::new("Total Score").fg(Color::White),
            Cell::new("Decision").fg(Color::White),
            Cell::new("Draw Result").fg(Color::Green),
        ]);

    for (rank, (id, rep)) in results.iter().enumerate() {
        let decision_cell = if rep.is_eligible() {
            Cell::new("QUALIFIED").fg(Color::Green)
        } else {
            Cell::new("DISQUALIFIED").fg(Color::Red)
        };

        let draw_cell = if !rep.is_eligible() {
            Cell::new("Ineligible").fg(Color::Red)
        } else if let Some(cut) = cutoff {
            if rep.total_score >= cut {
                Cell::new("SELECTED (ITA)").fg(Color::Green)
            } else {
                Cell::new("Below Cutoff").fg(Color::Yellow)
            }
        } else {
            Cell::new("Ranked").fg(Color::White)
        };

        table.add_row(vec![
            Cell::new((rank + 1).to_string()),
            Cell::new(id),
            Cell::new(format!("{:.1}", rep.total_score)).fg(Color::Cyan),
            decision_cell,
            draw_cell,
        ]);
    }

    println!("{table}\n");
}

fn run_benchmark(ruleset: &LoadedRuleset, iterations: usize) {
    let dummy_json = serde_json::json!({
        "applicant": {
            "id": "BENCH-001",
            "age": 28,
            "marital_status": "single",
            "job_offer": {
                "has_offer": true,
                "annual_salary": 45000.0,
                "rqf_skill_level": 4,
                "meets_occupation_going_rate": true,
                "sponsor": {
                    "is_licensed": true,
                    "license_rating": "A_rated"
                }
            },
            "language": {
                "cefr_level": "B2"
            }
        }
    });

    let facts = json_to_facts(&dummy_json);
    println!("\n{}", " ⚡ RUNNING ENGINE BENCHMARK ⚡ ".bold().on_yellow().black());
    println!(" Iterations:   {}", iterations.to_string().cyan());
    println!(" Ruleset:      {}", ruleset.kb.name().yellow());

    // Warm-up
    for _ in 0..10 {
        let _ = evaluate_facts(&ruleset.kb, &facts, ruleset.pass_mark);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = evaluate_facts(&ruleset.kb, &facts, ruleset.pass_mark);
    }
    let elapsed = start.elapsed();
    let total_secs = elapsed.as_secs_f64();
    let eps = (iterations as f64) / total_secs;
    let mean_us = (total_secs / (iterations as f64)) * 1_000_000.0;

    println!(" Total Time:   {:.3} seconds", total_secs);
    println!(" Throughput:   {} evals/sec", format!("{:.0}", eps).green().bold());
    println!(" Mean Latency: {} µs/eval\n", format!("{:.2}", mean_us).cyan().bold());
}
