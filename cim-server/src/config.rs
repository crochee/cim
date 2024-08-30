use std::{fs, ops::RangeInclusive};

use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug, Clone, Deserialize)]
#[command(name = "server")]
#[command(author, version, about, long_about = None)]
pub struct AppConfig {
    #[clap(long)]
    #[arg(short = 'c')]
    #[serde(default)]
    pub config: Option<String>,
    #[clap(long, env)]
    pub database_url: String,
    #[clap(long, env)]
    pub max_size: u32,
    #[clap(long, env)]
    pub min_idle: u32,
    #[clap(long, env)]
    #[serde(default)]
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
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    #[clap(long, env)]
    #[arg(default_value_t = 3600)]
    #[serde(default = "default_expiration")]
    pub expiration: i64,
    #[clap(long, env)]
    #[arg(default_value_t = 10)]
    #[serde(default = "default_time")]
    pub absolute_lifetime: i64,
    #[clap(long, env)]
    #[arg(default_value_t = 10)]
    #[serde(default = "default_time")]
    pub valid_if_not_used_for: i64,
    #[clap(long, env)]
    #[arg(default_value_t = 10)]
    #[serde(default = "default_time")]
    pub reuse_interval: i64,
    #[clap(long, env)]
    #[arg(default_value_t = true)]
    #[serde(default = "default_rotate")]
    pub rotate_refresh_tokens: bool,
}

fn default_endpoint() -> String {
    String::from("127.0.0.1")
}

fn default_expiration() -> i64 {
    3600
}

fn default_time() -> i64 {
    10
}

fn default_rotate() -> bool {
    true
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

pub fn load(cfg: &str) -> Result<AppConfig> {
    let content =
        fs::read_to_string(cfg).context("could not read config file")?;
    toml::from_str(&content).context("could not parse config file")
}
