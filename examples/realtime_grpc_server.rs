use colored::Colorize;
use rust_rules_engine::{AppState, RuleProgram, RulesGrpcService, RulesServiceServer};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules_dir = "rules/canada_crs/";
    let port = 50051;
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    println!(
        "\n{}",
        " ⚡ REAL-TIME RULES ENGINE gRPC MICROSERVICE ⚡ "
            .bold()
            .on_green()
            .black()
    );
    let program = RuleProgram::from_path(rules_dir)?;
    println!(
        "  • Ruleset Loaded:  {} (v{}) [{} rules]",
        program.name.cyan().bold(),
        program.version,
        program.rules.len()
    );
    println!(
        "  • gRPC Service:    {}",
        format!("http://localhost:{}", port).yellow().bold()
    );
    println!("  • Protocol:        HTTP/2 (Binary Protocol Buffers)");
    println!("  • RPC Methods:");
    println!("     rpc Evaluate (EvaluateRequest) returns (EvaluateResponse);");
    println!("     rpc BatchEvaluate (BatchEvaluateRequest) returns (BatchEvaluateResponse);");
    println!("     rpc SimulateWhatIf (WhatIfRequest) returns (WhatIfResponse);\n");

    let state = AppState::new(program);
    let grpc_service = RulesGrpcService::new(state);

    tonic::transport::Server::builder()
        .add_service(RulesServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
