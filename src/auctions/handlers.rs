use actix_web::{web, HttpResponse, post, put, delete, get};
use sqlx::{PgPool, Postgres, FromRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

use crate::error::AppError;

// --- Struct Definitions ---
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Auction {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub starting_bid: f64, // Corrected from starting_price
    pub current_highest_bid: Option<f64>, // Matches nullable DECIMAL
    pub highest_bidder_id: Option<Uuid>, // Matches nullable UUID
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bid {
    pub id: Uuid,
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    pub bid_amount: f64,
    pub bid_time: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuctionPayload {
    pub vehicle_id: Uuid,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub starting_bid: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAuctionPayload {
    pub current_highest_bid: Option<f64>,
    pub highest_bidder_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBidPayload {
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    pub amount: f64,
}

// --- Handler Functions ---

/// Handles creating a new bid and updating the auction's highest bid.
#[post("/bids")]
pub async fn create_bid(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateBidPayload>,
) -> Result<HttpResponse, AppError> {
    let now = OffsetDateTime::now_utc();

    let new_bid = sqlx::query_as!(
        Bid,
        r#"
        INSERT INTO bids (auction_id, bidder_id, bid_amount, bid_time)
        VALUES ($1, $2, $3, $4)
        RETURNING id, auction_id, bidder_id, bid_amount, bid_time
        "#,
        payload.auction_id,
        payload.bidder_id,
        payload.amount,
        now
    )
    .fetch_one(&**pool)
    .await?;

    // Update the auction's highest bid if the new bid is higher
    sqlx::query!(
        r#"
        UPDATE auctions
        SET current_highest_bid = $1,
            highest_bidder_id = $2,
            updated_at = $3
        WHERE id = $4 AND ($1 > current_highest_bid OR current_highest_bid IS NULL)
        "#,
        new_bid.bid_amount,
        new_bid.bidder_id,
        OffsetDateTime::now_utc(),
        payload.auction_id
    )
    .execute(&**pool)
    .await?;

    Ok(HttpResponse::Created().json(new_bid))
}

/// Handles creating a new auction.
#[post("/auctions")]
pub async fn create_auction(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateAuctionPayload>,
) -> Result<HttpResponse, AppError> {
    let now = OffsetDateTime::now_utc();

    let new_auction = sqlx::query_as!(
        Auction,
        r#"
        INSERT INTO auctions (vehicle_id, start_time, end_time, starting_bid, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, vehicle_id, start_time, end_time, starting_bid, current_highest_bid, highest_bidder_id, status, created_at, updated_at
        "#,
        payload.vehicle_id,
        payload.start_time,
        payload.end_time,
        payload.starting_bid,
        "active", // Default status for new auctions
        now,
        now
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Created().json(new_auction))
}

/// Handles updating an existing auction.
#[put("/auctions/{id}")]
pub async fn update_auction(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    payload: web::Json<UpdateAuctionPayload>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let now = OffsetDateTime::now_utc();

    let mut query_builder: sqlx::QueryBuilder<Postgres> =
        sqlx::QueryBuilder::new("UPDATE auctions SET updated_at = ");
    query_builder.push_bind(now);

    if let Some(current_highest_bid) = payload.current_highest_bid {
        query_builder.push(", current_highest_bid = ");
        query_builder.push_bind(current_highest_bid);
    }
    if let Some(highest_bidder_id) = payload.highest_bidder_id {
        query_builder.push(", highest_bidder_id = ");
        query_builder.push_bind(highest_bidder_id);
    }
    if let Some(status) = payload.status.clone() {
        query_builder.push(", status = ");
        query_builder.push_bind(status);
    }

    query_builder.push(" WHERE id = ");
    query_builder.push_bind(id);
    query_builder.push(" RETURNING id, vehicle_id, start_time, end_time, starting_bid, current_highest_bid, highest_bidder_id, status, created_at, updated_at");


    let updated_auction = query_builder
        .build_query_as::<Auction>()
        .fetch_one(&**pool)
        .await?;

    Ok(HttpResponse::Ok().json(updated_auction))
}

/// Handles deleting an auction.
#[delete("/auctions/{id}")]
pub async fn delete_auction(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let deleted_rows = sqlx::query!("DELETE FROM auctions WHERE id = $1", id)
        .execute(&**pool)
        .await?
        .rows_affected();

    if deleted_rows == 0 {
        return Err(AppError::NotFound(format!("Auction with id {} not found", id)));
    }

    Ok(HttpResponse::NoContent().finish())
}

/// Handles fetching all auctions.
#[get("/auctions")]
pub async fn get_all_auctions(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let auctions = sqlx::query_as!(
        Auction,
        r#"SELECT id, vehicle_id, start_time, end_time, starting_bid, current_highest_bid, highest_bidder_id, status, created_at, updated_at FROM auctions ORDER BY created_at DESC"#
    )
    .fetch_all(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(auctions))
}

/// Handles fetching a single auction by ID.
#[get("/auctions/{id}")]
pub async fn get_auction_detail(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let auction = sqlx::query_as!(
        Auction,
        r#"SELECT id, vehicle_id, start_time, end_time, starting_bid, current_highest_bid, highest_bidder_id, status, created_at, updated_at FROM auctions WHERE id = $1"#,
        id
    )
    .fetch_optional(&**pool)
    .await?;

    match auction {
        Some(a) => Ok(HttpResponse::Ok().json(a)),
        None => Err(AppError::NotFound(format!("Auction with id {} not found", id))),
    }
}

/// Handles fetching all bids for a specific auction.
#[get("/auctions/{id}/bids")]
pub async fn get_bids_for_auction(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let auction_id = path.into_inner();

    let bids = sqlx::query_as!(
        Bid,
        r#"SELECT id, auction_id, bidder_id, bid_amount, bid_time FROM bids WHERE auction_id = $1 ORDER BY bid_time ASC"#,
        auction_id
    )
    .fetch_all(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(bids))
}