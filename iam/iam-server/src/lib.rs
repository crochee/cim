mod app;
mod config;
mod controllers;
mod middlewares;
mod models;
mod pkg;
mod routes;
mod services;
mod store;
mod var;

use tikv_jemallocator::Jemalloc;

#[global_allocator]
static ALLOC: Jemalloc = Jemalloc;

pub use app::{App, AppState};
pub use config::AppConfig;
pub use routes::AppRouter;
pub use store::pool::connection_manager;
