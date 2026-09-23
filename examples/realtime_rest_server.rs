use colored::Colorize;
use rust_rules_engine::{AppState, create_rest_router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules_path = "rules/uk_skilled_worker_points.grl";
    let port = 8080;
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    println!(
        "\n{}",
        " 🌐 REAL-TIME RULES ENGINE REST MICROSERVICE (Powered by KSD-CO/rust-rule-engine) 🌐 "
            .bold()
            .on_blue()
            .white()
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
        "  • Listening On:    {}",
        format!("http://localhost:{}", port).yellow().bold()
    );
    println!("  • Endpoints:");
    println!("     POST /api/v1/evaluate  - Evaluate single candidate facts (microsecond latency)");
    println!("     POST /api/v1/batch     - Batch evaluate and rank candidates for draw selection");
    println!("     POST /api/v1/simulate  - Run What-If counterfactual pathway simulations");
    println!("     GET  /api/v1/inspect   - Inspect active ruleset metadata & caps");
    println!("     GET  /health           - Service health status check\n");

    let app = create_rest_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
