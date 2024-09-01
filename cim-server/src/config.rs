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
    #[arg(default_value_t = 50)]
    #[serde(default = "default_max_size")]
    pub max_size: u32,
    #[clap(long, env)]
    #[arg(default_value_t = 30)]
    #[serde(default = "default_min_idle")]
    pub min_idle: u32,
    #[clap(long, env)]
    #[arg(default_value_t = false)]
    #[serde(default)]
    pub run_migrations: bool,
    #[clap(long, env)]
    #[arg(default_value_t = String::from("server=info"))]
    #[serde(default = "default_rust_log")]
    pub rust_log: String,
    #[clap(long, env)]
    #[arg(value_parser = port_in_range,short = 'p', default_value_t = 30050)]
    #[serde(default = "default_port")]
    pub port: u16,
    #[clap(long, env)]
    pub cors_origin: String,
    #[clap(long, env)]
    #[arg(default_value_t = 512)]
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
    #[clap(long, env)]
    #[arg(default_value_t = String::from("0.0.0.0"))]
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

fn default_rust_log() -> String {
    String::from("server=info")
}

fn default_endpoint() -> String {
    String::from("0.0.0.0")
}

fn default_port() -> u16 {
    30050
}

fn default_max_size() -> u32 {
    50
}

fn default_min_idle() -> u32 {
    30
}

fn default_cache_size() -> usize {
    512
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
