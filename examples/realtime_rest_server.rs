use colored::Colorize;
use rust_rules_engine::{AppState, RuleProgram, create_rest_router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules_dir = "rules/canada_crs/";
    let port = 8080;
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    println!(
        "\n{}",
        " 🌐 REAL-TIME RULES ENGINE REST MICROSERVICE 🌐 "
            .bold()
            .on_blue()
            .white()
    );
    let program = RuleProgram::from_path(rules_dir)?;
    println!(
        "  • Ruleset Loaded:  {} (v{}) [{} rules]",
        program.name.cyan().bold(),
        program.version,
        program.rules.len()
    );
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

    let state = AppState::new(program);
    let app = create_rest_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
