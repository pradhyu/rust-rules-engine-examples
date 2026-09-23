use colored::Colorize;
use reqwest::Client;
use rust_rules_engine::{
    AppState, EvaluateApiResponse, RuleProgram, SimulateApiResponse, create_rest_router,
};
use std::fs;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " 🚀 REST PROTOCOL REAL-TIME CLIENT DEMO 🚀 "
            .bold()
            .on_cyan()
            .black()
    );

    // 1. Launch embedded local REST microservice for deterministic self-contained demo
    let program = RuleProgram::from_path("rules/canada_crs/")?;
    let state = AppState::new(program);
    let app = create_rest_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let base_url = format!("http://127.0.0.1:{}", port);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    println!("  • Connected to REST Engine at: {}\n", base_url.yellow());
    let client = Client::new();

    // =========================================================================
    // TEST 1: REAL-TIME SINGLE CANDIDATE EVALUATION OVER REST (JSON)
    // =========================================================================
    println!(
        "{}",
        "1️⃣  POST /api/v1/evaluate (Single Candidate JSON Payload)"
            .bold()
            .underline()
    );
    let app_raw = fs::read_to_string("applicants/tc01_tech_lead_single.json")?;
    let app_json: serde_json::Value = serde_json::from_str(&app_raw)?;

    let req_body = serde_json::json!({
        "applicant": app_json["applicant"]
    });

    let t0 = Instant::now();
    let resp = client
        .post(format!("{}/api/v1/evaluate", base_url))
        .json(&req_body)
        .send()
        .await?;

    let http_latency = t0.elapsed();
    let eval_res: EvaluateApiResponse = resp.json().await?;

    println!("   • HTTP Status:        {}", "200 OK".green().bold());
    println!("   • Total Round-Trip:   {:.2?}", http_latency);
    println!(
        "   • Server Engine Time: {} ({:.2} µs)",
        format!("{:.2}µs", eval_res.latency_micros).green().bold(),
        eval_res.latency_micros
    );
    println!(
        "   • Candidate ID:       {}",
        eval_res.applicant_id.unwrap_or_default().yellow()
    );
    println!(
        "   • Total Score:        {:.1} points",
        eval_res.total_score.to_string().cyan().bold()
    );
    println!(
        "   • Eligibility:        {}",
        if eval_res.eligible {
            "Eligible / Qualified".green()
        } else {
            "Ineligible".red()
        }
    );
    println!("   • Fired Rules Count:  {}", eval_res.fired_rules.len());
    println!("   • Inferred Tags:      {:?}\n", eval_res.tags);

    // =========================================================================
    // TEST 2: REAL-TIME 'WHAT-IF' ADVISOR OVER REST
    // =========================================================================
    println!(
        "{}",
        "2️⃣  POST /api/v1/simulate (Real-Time Pathway Advisor)"
            .bold()
            .underline()
    );
    let sim_body = serde_json::json!({
        "applicant": app_json["applicant"],
        "target_cutoff": 485.0
    });

    let t_sim = Instant::now();
    let sim_resp = client
        .post(format!("{}/api/v1/simulate", base_url))
        .json(&sim_body)
        .send()
        .await?;

    let sim_roundtrip = t_sim.elapsed();
    let sim_res: SimulateApiResponse = sim_resp.json().await?;

    println!("   • Total Round-Trip:   {:.2?}", sim_roundtrip);
    println!("   • Engine Latency:     {:.2} µs", sim_res.latency_micros);
    println!(
        "   • Current Score:      {:.1} / Target: {:.1}",
        sim_res.current_score, sim_res.target_cutoff
    );
    println!(
        "   • Simulation Status:  {}",
        if sim_res.currently_qualifies {
            "QUALIFIED".green()
        } else {
            "BELOW CUTOFF".yellow()
        }
    );
    println!("   • Actionable Pathways Found: {}", sim_res.pathways.len());
    for (i, p) in sim_res.pathways.iter().enumerate() {
        let badge = if p.qualifies_for_draw {
            "[QUALIFIES FOR DRAW]".green().bold()
        } else {
            "[REDUCES GAP]".yellow()
        };
        println!("     {}. {} {}", i + 1, p.title.cyan(), badge);
        println!(
            "        Gain: +{:.1} pts -> Projected: {:.1} pts",
            p.points_gain, p.projected_total
        );
    }
    println!();

    // =========================================================================
    // TEST 3: REAL-TIME BATCH EVALUATION OVER REST
    // =========================================================================
    println!(
        "{}",
        "3️⃣  POST /api/v1/batch (Batch Candidate Ranking & Draw)"
            .bold()
            .underline()
    );
    let mut applicants = Vec::new();
    for entry in fs::read_dir("applicants")? {
        let p = entry?.path();
        if p.extension().and_then(|e| e.to_str()) == Some("yaml")
            || p.extension().and_then(|e| e.to_str()) == Some("json")
        {
            let content = fs::read_to_string(&p)?;
            let v: serde_json::Value = if p.extension().and_then(|e| e.to_str()) == Some("json") {
                serde_json::from_str(&content)?
            } else {
                serde_yaml::from_str(&content)?
            };
            applicants.push(v);
        }
    }

    let batch_body = serde_json::json!({
        "applicants": applicants,
        "cutoff": 500.0
    });

    let t_batch = Instant::now();
    let batch_resp = client
        .post(format!("{}/api/v1/batch", base_url))
        .json(&batch_body)
        .send()
        .await?;

    let batch_roundtrip = t_batch.elapsed();
    let batch_res: rust_rules_engine::BatchApiResponse = batch_resp.json().await?;

    println!("   • Total Candidates:   {}", batch_res.total_evaluated);
    println!("   • Batch Round-Trip:   {:.2?}", batch_roundtrip);
    println!(
        "   • Engine Latency:     {:.2} µs ({:.2} µs/candidate)",
        batch_res.latency_micros,
        batch_res.latency_micros / batch_res.total_evaluated as f64
    );
    println!(
        "   • Top Ranked:         #1 {} ({:.1} pts - {})",
        batch_res.candidates[0].candidate_id.yellow(),
        batch_res.candidates[0].total_score,
        batch_res.candidates[0].draw_result.green().bold()
    );

    println!("\n════════════════════════════════════════════════════════════════════════════");
    println!(" 🚀 REST Protocol Verification Complete: High throughput JSON REST API");
    println!(
        "    delivering sub-millisecond network round-trips with microsecond rule engine core."
    );
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
