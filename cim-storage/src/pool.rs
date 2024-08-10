use anyhow::Context;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tracing::info;

pub async fn connection_manager(
    uri: &str,
    max_size: u32,
    min_idle: u32,
    run_migrations: bool,
) -> anyhow::Result<MySqlPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(max_size)
        .min_connections(min_idle)
        .connect(uri)
        .await
        .context("error while initializing the database connection pool")?;

    if run_migrations {
        info!("migrations enabled, running...");
        sqlx::migrate!()
            .run(&pool)
            .await
            .context("error while running database migrations")?;
    }

    Ok(pool)
}
