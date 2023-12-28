use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tokio::{net::TcpListener, runtime::Builder, signal};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cims::{connection_manager, version, App, AppConfig, AppRouter, AppState};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let config = AppConfig::parse();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("{}", version());
    // tokio的运行时配置
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("could not initialize multi-thread runtime")?
        .block_on(async move { async_run_server(config).await })
}

async fn async_run_server(config: AppConfig) -> anyhow::Result<()> {
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

    let router = AppRouter::build(&config.cors_origin, AppState(app))
        .context("could not initialize application routes")?;
    let host = format!("{}:{}", config.endpoint, config.port);
    info!("routes initialized, listening on {}", host);
    let listener = TcpListener::bind(host)
        .await
        .context("could not bind to endpoint")?;

    axum::serve(listener, router.into_make_service())
        // .with_graceful_shutdown(shutdown_signal())
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

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
