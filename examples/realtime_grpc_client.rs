use colored::Colorize;
use rust_rules_engine::proto::rules_service_client::RulesServiceClient;
use rust_rules_engine::proto::{BatchEvaluateRequest, EvaluateRequest, WhatIfRequest};
use rust_rules_engine::{AppState, RuleProgram, RulesGrpcService, RulesServiceServer};
use std::fs;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        " ⚡ gRPC PROTOCOL REAL-TIME CLIENT DEMO (HTTP/2 + Protobuf) ⚡ "
            .bold()
            .on_green()
            .black()
    );

    // 1. Launch embedded local gRPC microservice on random available port
    let program = RuleProgram::from_path("rules/canada_crs/")?;
    let state = AppState::new(program);
    let grpc_service = RulesGrpcService::new(state);

    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let local_addr = listener.local_addr()?;
    drop(listener); // release port for tonic server to bind
    let server_url = format!("http://127.0.0.1:{}", local_addr.port());

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(RulesServiceServer::new(grpc_service))
            .serve(local_addr)
            .await
            .unwrap();
    });

    // Small delay to ensure server starts
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    println!("  • Connected to gRPC Engine at: {}\n", server_url.yellow());
    let mut client = RulesServiceClient::connect(server_url).await?;

    // =========================================================================
    // TEST 1: REAL-TIME SINGLE CANDIDATE EVALUATION OVER gRPC
    // =========================================================================
    println!(
        "{}",
        "1️⃣  rpc Evaluate (Protobuf Payload over HTTP/2)"
            .bold()
            .underline()
    );
    let app_raw = fs::read_to_string("applicants/tc01_tech_lead_single.json")?;

    let req = tonic::Request::new(EvaluateRequest {
        rules_path: String::new(),
        applicant_json: app_raw.clone(),
    });

    let t0 = Instant::now();
    let resp = client.evaluate(req).await?.into_inner();
    let grpc_latency = t0.elapsed();

    println!("   • gRPC Status:        {}", "OK (Code 0)".green().bold());
    println!("   • Total Round-Trip:   {:.2?}", grpc_latency);
    println!(
        "   • Server Engine Time: {} ({:.2} µs)",
        format!("{:.2}µs", resp.latency_micros).green().bold(),
        resp.latency_micros
    );
    println!(
        "   • Total Score:        {:.1} points",
        resp.total_score.to_string().cyan().bold()
    );
    println!(
        "   • Eligibility:        {}",
        if resp.is_eligible {
            "Eligible / Qualified".green()
        } else {
            "Ineligible".red()
        }
    );
    println!("   • Fired Rules Count:  {}", resp.fired_rules.len());
    println!("   • Category Breakdown:");
    for cat in &resp.categories {
        println!(
            "      - {:<32} -> {:.1} / {:.1} pts",
            cat.display_name, cat.capped_score, cat.max_points
        );
    }
    println!();

    // =========================================================================
    // TEST 2: REAL-TIME WHAT-IF SIMULATION OVER gRPC
    // =========================================================================
    println!(
        "{}",
        "2️⃣  rpc SimulateWhatIf (Real-Time Pathway Advisor)"
            .bold()
            .underline()
    );
    let whatif_req = tonic::Request::new(WhatIfRequest {
        rules_path: String::new(),
        applicant_json: app_raw,
        target_cutoff: 485.0,
    });

    let t_sim = Instant::now();
    let whatif_res = client.simulate_what_if(whatif_req).await?.into_inner();
    let whatif_latency = t_sim.elapsed();

    println!("   • Total Round-Trip:   {:.2?}", whatif_latency);
    println!(
        "   • Current Score:      {:.1} / Target Cutoff: {:.1}",
        whatif_res.current_score, whatif_res.target_cutoff
    );
    println!(
        "   • Status:             {}",
        if whatif_res.currently_qualifies {
            "QUALIFIED".green()
        } else {
            "BELOW CUTOFF".yellow()
        }
    );
    println!(
        "   • Actionable Pathways ({} found):",
        whatif_res.pathways.len()
    );
    for (i, p) in whatif_res.pathways.iter().enumerate() {
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
    // TEST 3: REAL-TIME BATCH EVALUATION OVER gRPC
    // =========================================================================
    println!(
        "{}",
        "3️⃣  rpc BatchEvaluate (Batch Candidate Ranking)"
            .bold()
            .underline()
    );
    let mut applicant_jsons = Vec::new();
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
            applicant_jsons.push(serde_json::to_string(&v)?);
        }
    }

    let batch_req = tonic::Request::new(BatchEvaluateRequest {
        rules_path: String::new(),
        applicant_jsons,
        cutoff_score: 500.0,
    });

    let t_batch = Instant::now();
    let batch_res = client.batch_evaluate(batch_req).await?.into_inner();
    let batch_latency = t_batch.elapsed();

    println!("   • Total Candidates:   {}", batch_res.total_evaluated);
    println!("   • Batch Round-Trip:   {:.2?}", batch_latency);
    println!(
        "   • Engine Latency:     {:.2} µs",
        batch_res.latency_micros
    );
    println!(
        "   • Top Ranked:         #1 {} ({:.1} pts - {})",
        batch_res.candidates[0].candidate_id.yellow(),
        batch_res.candidates[0].total_score,
        batch_res.candidates[0].draw_status.green().bold()
    );

    println!("\n════════════════════════════════════════════════════════════════════════════");
    println!(" 🚀 gRPC Protocol Verification Complete: Binary Protobuf over HTTP/2");
    println!("    delivering minimum wire overhead and ultra-fast microsecond rule execution.");
    println!("════════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
