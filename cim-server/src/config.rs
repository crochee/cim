use std::ops::RangeInclusive;

use clap::Parser;

#[derive(Parser, Debug, Clone)] // requires `derive` feature
#[command(name = "server")]
#[command(author, version, about, long_about = None)]
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
    #[arg(value_parser = port_in_range,short = 'p')]
    pub port: u16,
    #[clap(long, env)]
    pub cors_origin: String,
    #[clap(long, env)]
    pub cache_size: usize,
    #[clap(long, env)]
    #[arg(default_value_t = String::from("127.0.0.1"))]
    pub endpoint: String,
    #[clap(long, env)]
    #[arg(default_value_t = 3600)]
    pub expiration: i64,
    #[clap(long, env)]
    #[arg(default_value_t = 10)]
    pub absolute_lifetime: i64,
    #[clap(long, env)]
    #[arg(default_value_t = 10)]
    pub valid_if_not_used_for: i64,
    #[clap(long, env)]
    #[arg(default_value_t = 10)]
    pub reuse_interval: i64,
    #[clap(long, env)]
    #[arg(default_value_t = true)]
    pub rotate_refresh_tokens: bool,
}

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a port number"))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}
