use axum::{Router, routing::post, extract::{Extension, Path, Json}, response::IntoResponse, http::StatusCode};
use serde::Deserialize;
use crate::db::DbPool;
use uuid::Uuid;

#[derive(Deserialize)]
struct BidPayload { bid_amount: f64 }

async fn place_bid(Path(auction_id): Path<Uuid>, Json(payload): Json<BidPayload>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let mut tx = pool.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // ensure auction exists and get current_highest_bid
    let current = sqlx::query!("SELECT current_highest_bid FROM auctions WHERE id=$1", auction_id)
        .fetch_one(&mut tx).await.map_err(|_| StatusCode::NOT_FOUND)?;
    let highest = current.current_highest_bid.unwrap_or(0.0);
    if payload.bid_amount <= highest { return Err(StatusCode::BAD_REQUEST); }
    // insert bid
    sqlx::query!("INSERT INTO bids(id, auction_id, amount, placed_at) VALUES($1,$2,$3,$4)", id, auction_id, payload.bid_amount, now)
        .execute(&mut tx).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    // update auction
    sqlx::query!("UPDATE auctions SET current_highest_bid=$1 WHERE id=$2", payload.bid_amount, auction_id)
        .execute(&mut tx).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    (StatusCode::CREATED, Json(id))
}

pub fn router() -> Router {
    Router::new().route("/auctions/:id/bid", post(place_bid))
}
