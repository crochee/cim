use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use clap::{Parser, Subcommand};
use tokio::{runtime::Builder, sync::oneshot};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cim_server::{connection_manager, App, AppConfig, AppRouter, AppState};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let args = Cli::parse();
    match args.command {
        Commands::Run(config) => {
            run_server(config)?;
        }
        Commands::Version => {
            println!("{}", cim_server::version());
            return Ok(());
        }
    }
    Ok(())
}

// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "server")]
#[command(author, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Run(AppConfig),
    #[command(short_flag = 'v')]
    Version,
}

fn run_server(config: AppConfig) -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

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
    axum::Server::bind(&SocketAddr::from((endpoint, port)))
        .http1_title_case_headers(true)
        .serve(router.into_make_service())
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
        })
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
