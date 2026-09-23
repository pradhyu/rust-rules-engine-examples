use colored::Colorize;
use rust_rules_engine::{AppState, RulesGrpcService, RulesServiceServer};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules_path = "rules/uk_skilled_worker_points.grl";
    let port = 50051;
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    println!(
        "\n{}",
        " ⚡ REAL-TIME RULES ENGINE gRPC MICROSERVICE (Powered by KSD-CO/rust-rule-engine) ⚡ "
            .bold()
            .on_green()
            .black()
    );
    let state = AppState::from_path(rules_path)?;
    let kb = state.kb.read().await;
    println!(
        "  • Knowledge Base:  {} [{} rules]",
        kb.name().cyan().bold(),
        kb.rule_count()
    );
    drop(kb);

    println!(
        "  • gRPC Service:    {}",
        format!("http://localhost:{}", port).yellow().bold()
    );
    println!("  • Protocol:        HTTP/2 (Binary Protocol Buffers)");
    println!("  • RPC Methods:");
    println!("     rpc Evaluate (EvaluateRequest) returns (EvaluateResponse);");
    println!("     rpc BatchEvaluate (BatchEvaluateRequest) returns (BatchEvaluateResponse);");
    println!("     rpc SimulateWhatIf (WhatIfRequest) returns (WhatIfResponse);\n");

    let grpc_service = RulesGrpcService::new(state);

    tonic::transport::Server::builder()
        .add_service(RulesServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
