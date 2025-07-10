use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use crate::config::get;

pub type DbPool = Arc<sqlx::PgPool>;

pub async fn init_db() -> DbPool {
    let pool = PgPoolOptions::new().max_connections(10).connect(&get().database_url).await.expect("DB connect failed");
    sqlx::migrate!().run(&pool).await.expect("Migrations failed");
    Arc::new(pool)
}

// Axum Extension extractor
pub use axum::extract::Extension;
