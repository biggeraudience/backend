#[macro_use]
extern crate lazy_static;

mod config;
mod db;
mod routes;
mod utils;

use axum::{Router, routing::get, AddExtensionLayer};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing/logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Initialize database (runs migrations)
    let pool = db::init_db().await;

    // Build CORS layer (allow all for now; adjust origins in production)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build application router
    let app = Router::new()
        .route("/healthz", get(routes::health::healthz))
        .merge(routes::auth::router())
        .merge(routes::vehicles::router())
        .merge(routes::auctions::router())
        .merge(routes::bids::router())
        .merge(routes::inquiries::router())
        .merge(routes::users::router())
        .layer(cors)
        .layer(AddExtensionLayer::new(pool));

    // Start server
    let addr = format!("0.0.0.0:{}", config::get().port);
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
