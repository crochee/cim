#[derive(clap::Parser)]
pub struct AppConfig {
    #[clap(long, env)]
    pub database_url: String,
    #[clap(long, env)]
    pub max_size: u32,
    #[clap(long, env)]
    pub min_idle: u32,
    #[clap(long, env)]
    pub run_migrations: bool,
    #[clap(long, env)]
    pub rust_log: String,
    #[clap(long, env)]
    pub port: u16,
    #[clap(long, env)]
    pub cors_origin: String,
}
