use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::config::Settings;

pub type DbPool = PgPool;

pub async fn create_pool(settings: &Settings) -> Result<DbPool> {
    // Use `connect_lazy` so creating the pool does not attempt an immediate
    // network connection. This allows the HTTP server (and `/health` probe)
    // to start even if the database is temporarily unavailable during
    // deployment. Actual queries will still try to connect when executed.
    let pool = PgPoolOptions::new()
        .max_connections(settings.database_max_connections)
        .min_connections(settings.database_min_connections)
        .acquire_timeout(Duration::from_secs(settings.database_connect_timeout_seconds))
        .connect_lazy(&settings.database_url)
        .with_context(|| "Failed to create lazy Postgres pool. Check DATABASE_URL format.")?;

    Ok(pool)
}
