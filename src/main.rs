#[macro_use]
extern crate lazy_static;

pub mod config;
pub mod db;
pub mod routes;
pub mod utils;

use axum::{Router, routing::get, extract::Extension};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let pool = db::init_db().await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/healthz", get(routes::health::healthz))
        .merge(routes::auth::router())
        .merge(routes::vehicles::router())
        .merge(routes::auctions::router())
        .merge(routes::bids::router())
        .merge(routes::inquiries::router())
        .merge(routes::users::router())
        .layer(cors)
        .layer(Extension(pool)); // ✅ updated

    let addr = format!("0.0.0.0:{}", config::get().port);
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
