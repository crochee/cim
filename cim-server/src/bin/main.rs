use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cim_storage::connection_manager;

use cim_server::{
    shutdown_signal, version, App, AppConfig, AppRouter, AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let config = AppConfig::parse();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("{}", version());
    async_run_server(config).await
}

async fn async_run_server(config: AppConfig) -> Result<()> {
    info!("environment loaded and configuration parsed, initializing Mariadb connection and running migrations...");
    let store = connection_manager(
        &config.database_url,
        config.max_size,
        config.min_idle,
        config.run_migrations,
    )
    .await
    .context("could not initialize the database connection pool")?;

    info!("migrations successfully run, initializing axum server...");

    let app = Arc::new(App::new(store, config.clone())?);

    key_rotate(app.clone());

    let router = AppRouter::build(AppState(app))
        .context("could not initialize application routes")?;
    let host = format!("{}:{}", config.endpoint, config.port);
    info!("routes initialized, listening on {}", host);
    let listener = TcpListener::bind(host)
        .await
        .context("could not bind to endpoint")?;

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("error while starting API server")?;

    Ok(())
}

fn key_rotate(app: Arc<App>) {
    tokio::spawn(async move {
        info!("start first rotate...");
        if let Err(err) = app.key_rotator.rotate().await {
            error!("{}", err);
        }
        info!("start first finish!");
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    info!("start rotate...");
                    if let Err(err) = app.key_rotator.rotate().await {
                        error!("{}", err);
                    }
                    info!("start rotate finish!");
                },
                _ = shutdown_signal() => {
                    break;
                }
            }
        }
    });
}
