use std::net::IpAddr;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tokio::{net::TcpListener, runtime::Builder, sync::oneshot};
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

    let port = config.port;
    let endpoint = config.endpoint.clone();
    let app = Arc::new(App::new(store, config.clone())?);

    key_rotate(app.clone());

    let router = AppRouter::build(&config.cors_origin, AppState(app))
        .context("could not initialize application routes")?;

    info!("routes initialized, listening on port {}", port);
    let listener = TcpListener::bind((
        endpoint
            .parse::<IpAddr>()
            .context("could not parse endpoint")?,
        port,
    ))
    .await
    .context("could not bind to endpoint")?;

    axum::serve(listener, router.into_make_service())
        .await
        .context("error while starting API server")?;

    Ok(())
}

fn key_rotate(app: Arc<App>) {
    tokio::spawn(async {
        let (tx, mut rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = tx.send(true);
        });
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
                    _ = &mut rx => {
                        break;
                    }
                }
            }
        });
    });
}
