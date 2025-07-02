// src/auctions/handlers.rs
use actix_web::{web, HttpResponse, post, put, delete, get};
use sqlx::{PgPool, Postgres, FromRow};
use uuid::Uuid;
use time::OffsetDateTime;
// Change this line:
use sqlx_types_bigdecimal::BigDecimal; // Use the BigDecimal from sqlx-types-bigdecimal

use crate::error::AppError;
use crate::auctions::models::{Auction, Bid}; // Import Auction and Bid from models
use crate::auctions::payloads::{CreateAuctionPayload, UpdateAuctionPayload, CreateBidPayload}; // Import payloads from payloads.rs

// --- Handler Functions ---

/// Handles creating a new bid and updating the auction's highest bid.
#[post("/bids")]
pub async fn create_bid(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateBidPayload>,
) -> Result<HttpResponse, AppError> {
    // Validate the payload using the `validator` crate
    payload.validate()?;

    // --- START: Added Pre-DB Bid Validation ---
    // Fetch the current auction details to validate the bid
    let auction_details = sqlx::query_as!(
        Auction,
        r#"SELECT id, vehicle_id, start_time, end_time, starting_bid, current_highest_bid, highest_bidder_id, status, created_at, updated_at FROM auctions WHERE id = $1"#,
        payload.auction_id
    )
    .fetch_optional(&**pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Auction not found".to_string()))?;

    // Check if the auction is active
    if auction_details.status != "active" {
        return Err(AppError::ValidationError(format!("Cannot place bid: Auction is currently in '{}' status.", auction_details.status).into()));
    }

    // Check if the bid amount is valid
    if let Some(current_bid) = auction_details.current_highest_bid {
        if payload.amount <= current_bid {
            return Err(AppError::ValidationError("Bid must be higher than the current highest bid.".into()));
        }
    } else {
        // If no bids yet, ensure it's at least the starting bid
        if payload.amount < auction_details.starting_bid {
            return Err(AppError::ValidationError(format!(
                "Bid must be at least the starting bid: {}",
                auction_details.starting_bid
            ).into()));
        }
    }
    // --- END: Added Pre-DB Bid Validation ---

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
    // The WHERE clause `$1 > current_highest_bid OR current_highest_bid IS NULL`
    // provides an additional database-level guard against race conditions, which is crucial.
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
    // Validate the payload using the `validator` crate
    payload.validate()?;

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
// Simplified to a fixed query_as! call as requested
#[put("/auctions/{id}")]
pub async fn update_auction(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    payload: web::Json<UpdateAuctionPayload>,
) -> Result<HttpResponse, AppError> {
    // Validate the payload using the `validator` crate
    payload.validate()?;

    let id = path.into_inner();
    let now = OffsetDateTime::now_utc();

    // Fetch existing auction details to apply partial updates or validation
    let mut existing_auction = sqlx::query_as!(
        Auction,
        r#"SELECT id, vehicle_id, start_time, end_time, starting_bid, current_highest_bid, highest_bidder_id, status, created_at, updated_at FROM auctions WHERE id = $1"#,
        id
    )
    .fetch_optional(&**pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Auction with id {} not found", id)))?;

    // Apply updates from payload, falling back to existing values if not provided
    existing_auction.current_highest_bid = payload.current_highest_bid.or(existing_auction.current_highest_bid);
    existing_auction.highest_bidder_id = payload.highest_bidder_id.or(existing_auction.highest_bidder_id);
    existing_auction.status = payload.status.unwrap_or(existing_auction.status);

    let updated_auction = sqlx::query_as!(
        Auction,
        r#"
        UPDATE auctions
        SET current_highest_bid = $1,
            highest_bidder_id = $2,
            status = $3,
            updated_at = $4
        WHERE id = $5
        RETURNING id, vehicle_id, start_time, end_time, starting_bid, current_highest_bid, highest_bidder_id, status, created_at, updated_at
        "#,
        existing_auction.current_highest_bid,
        existing_auction.highest_bidder_id,
        existing_auction.status,
        now,
        id
    )
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