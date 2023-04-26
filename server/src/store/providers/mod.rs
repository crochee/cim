mod mariadb;

use serde::Deserialize;

pub use mariadb::*;

#[derive(Debug, Deserialize)]
pub struct Content {
    pub secret: String,
    pub redirect_url: String,
    pub name: String,
    pub prompt: String,
    pub logo_url: String,
    pub refresh: bool,
}
