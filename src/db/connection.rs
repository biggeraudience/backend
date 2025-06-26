// src/db/connection.rs
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

pub async fn get_connection_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Connecting to {}", database_url);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    info!("✅ DB connected");
    Ok(pool)
}