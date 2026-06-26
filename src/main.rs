use tokio::net::TcpListener;
use rust_agent::app::{AppState, build_router};
use rust_agent::config::app::AppConfig;
use rust_agent::logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    logger::init();

    let config = AppConfig::from_env()?;
    let bind_addr = config.server.bind_addr();
    let app = build_router(AppState::new(config));
    let listener = TcpListener::bind(&bind_addr).await?;

    tracing::info!("listening on {}", bind_addr);
    axum::serve(listener, app)
        .await?;
    Ok(())
}
