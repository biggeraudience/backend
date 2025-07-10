use axum::{Router, routing::{get, post, put, delete}, extract::{Extension, Path, Json}, response::IntoResponse, http::StatusCode};
use serde::{Serialize, Deserialize};
use crate::db::DbPool;
use uuid::Uuid;

#[derive(Serialize)]
struct Auction { id: Uuid, vehicle_id: Uuid, start_time: chrono::DateTime<chrono::Utc>, end_time: chrono::DateTime<chrono::Utc>, starting_bid: f64, current_highest_bid: Option<f64>, status: String }
#[derive(Deserialize)]
struct AuctionPayload { vehicle_id: Uuid, start_time: chrono::DateTime<chrono::Utc>, end_time: chrono::DateTime<chrono::Utc>, starting_bid: f64, status: String }

async fn list_auctions(Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    let recs = sqlx::query_as!(Auction, "SELECT id, vehicle_id, start_time, end_time, starting_bid, current_highest_bid, status FROM auctions ORDER BY start_time DESC")
        .fetch_all(&*pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    (StatusCode::OK, Json(recs))
}

async fn get_auction(Path(id): Path<Uuid>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    let rec = sqlx::query_as!(Auction, "SELECT id, vehicle_id, start_time, end_time, starting_bid, current_highest_bid, status FROM auctions WHERE id=$1", id)
        .fetch_one(&*pool).await.map_err(|_| StatusCode::NOT_FOUND)?;
    (StatusCode::OK, Json(rec))
}

async fn create_auction(Json(payload): Json<AuctionPayload>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    let id = Uuid::new_v4();
    sqlx::query!("INSERT INTO auctions(id, vehicle_id, start_time, end_time, starting_bid, status) VALUES($1,$2,$3,$4,$5,$6)",
        id, payload.vehicle_id, payload.start_time, payload.end_time, payload.starting_bid, payload.status)
        .execute(&*pool).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    (StatusCode::CREATED, Json(id))
}

async fn update_auction(Path(id): Path<Uuid>, Json(payload): Json<AuctionPayload>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    sqlx::query!("UPDATE auctions SET vehicle_id=$1, start_time=$2, end_time=$3, starting_bid=$4, status=$5 WHERE id=$6",
        payload.vehicle_id, payload.start_time, payload.end_time, payload.starting_bid, payload.status, id)
        .execute(&*pool).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    StatusCode::NO_CONTENT
}

async fn delete_auction(Path(id): Path<Uuid>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    sqlx::query!("DELETE FROM auctions WHERE id=$1", id)
        .execute(&*pool).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    StatusCode::NO_CONTENT
}

pub fn router() -> Router {
    Router::new()
        .route("/auctions", get(list_auctions).post(create_auction))
        .route("/auctions/:id", get(get_auction).put(update_auction).delete(delete_auction))
}
