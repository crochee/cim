mod app;
mod config;
mod controllers;
mod middlewares;
mod routes;
mod services;
mod valid;
mod var;
mod version;

// #[cfg(all(feature = "mimalloc"))]
// #[global_allocator]
// static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;
//
// #[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
// #[global_allocator]
// static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub use app::{App, AppState};
pub use config::AppConfig;
pub use routes::AppRouter;
pub use version::version;
