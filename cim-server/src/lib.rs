mod app;
mod config;
mod controllers;
mod middlewares;
mod routes;
mod services;
mod valid;
mod var;
mod version;

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
pub use version::version;
