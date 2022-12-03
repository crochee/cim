use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use iam_server::{
    connection_manager, AppConfig, ApplicationController, ServiceRegister,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("environment loaded and configuration parsed, initializing Mariadb connection and running migrations...");
    let pool = connection_manager(
        &config.database_url,
        config.max_size,
        config.min_idle,
        config.run_migrations,
    )
    .await
    .context("could not initialize the database connection pool")?;

    let port = config.port;
    let service_register = ServiceRegister::new(pool, config.clone());

    info!("migrations successfully ran, initializing axum server...");
    ApplicationController::serve(port, &config.cors_origin, service_register)
        .await
        .context("could not initialize application routes")?;

    Ok(())
}
