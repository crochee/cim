mod app;
mod config;
mod controllers;
mod middlewares;
mod routes;
mod services;
mod valid;
mod var;
mod version;
mod auth;

#[cfg(target_env = "msvc")]
#[global_allocator]
#[cfg(target_env = "msvc")]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
#[cfg(not(target_env = "msvc"))]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub use app::{App, AppState};
pub use config::AppConfig;
pub use routes::AppRouter;
use tokio::signal;
pub use version::version;

pub async fn shutdown_signal() {
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
