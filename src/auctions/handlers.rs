// src/auctions/handlers.rs
use actix_web::{get, post, put, delete, web, HttpResponse};
use sqlx::PgPool;
use web::{Data, Json};
use uuid::Uuid;
use chrono::Utc;

use crate::auth::models::Claims;
use crate::error::AppError;
use crate::auctions::models::{
    Auction, Bid, CreateAuctionPayload, UpdateAuctionPayload, PlaceBidPayload,
};

// Public Endpoints
#[get("/")]
pub async fn get_all_auctions(pool: Data<PgPool>) -> Result<HttpResponse, AppError> {
    let auctions: Vec<Auction> = sqlx::query_as_unchecked!(
        Auction,
        r#"
        SELECT id, vehicle_id, start_time, end_time, starting_bid,
               current_highest_bid, highest_bidder_id, status,
               created_at, updated_at
        FROM auctions
        WHERE status = 'active'
        ORDER BY start_time ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(auctions))
}

#[get("/{auction_id}")]
pub async fn get_auction_detail(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let auction_id = path.into_inner();

    let auction: Auction = sqlx::query_as_unchecked!(
        Auction,
        r#"
        SELECT id, vehicle_id, start_time, end_time, starting_bid,
               current_highest_bid, highest_bidder_id, status,
               created_at, updated_at
        FROM auctions
        WHERE id = $1
        "#,
        auction_id
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Auction".to_string()))?;

    Ok(HttpResponse::Ok().json(auction))
}

#[post("/{auction_id}/bid")]
pub async fn place_bid(
    path: web::Path<Uuid>,
    claims: Claims,
    pool: Data<PgPool>,
    payload: Json<PlaceBidPayload>,
) -> Result<HttpResponse, AppError> {
    let auction_id = path.into_inner();
    let bid_amount = payload.bid_amount;

    let mut tx = pool.begin().await?;

    let auction: Auction = sqlx::query_as_unchecked!(
        Auction,
        r#"
        SELECT id, vehicle_id, start_time, end_time, starting_bid,
               current_highest_bid, highest_bidder_id, status,
               created_at, updated_at
        FROM auctions
        WHERE id = $1 FOR UPDATE
        "#,
        auction_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Auction".to_string()))?;

    if auction.status != "active" || Utc::now() < auction.start_time || Utc::now() > auction.end_time {
        return Err(AppError::ValidationError("Auction is not active or has ended.".into()));
    }

    let min_bid = auction.current_highest_bid.unwrap_or(auction.starting_bid);
    if bid_amount <= min_bid {
        return Err(AppError::ValidationError(
            format!("Bid must be higher than current highest bid ({:.2}).", min_bid),
        ));
    }

    if let Some(current) = auction.highest_bidder_id {
        if current == claims.user_id {
            return Err(AppError::ValidationError("You are already the highest bidder.".into()));
        }
    }

    let new_bid: Bid = sqlx::query_as_unchecked!(
        Bid,
        r#"
        INSERT INTO bids (auction_id, bidder_id, bid_amount, bid_time)
        VALUES ($1, $2, $3, $4)
        RETURNING id, auction_id, bidder_id, bid_amount, bid_time
        "#,
        auction_id,
        claims.user_id,
        bid_amount,
        Utc::now()
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query_unchecked!(
        r#"
        UPDATE auctions
        SET current_highest_bid = $1,
            highest_bidder_id = $2
        WHERE id = $3
        "#,
        bid_amount,
        claims.user_id,
        auction_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Created().json(new_bid))
}

// Admin Endpoints
#[post("/")]
pub async fn create_auction(
    pool: Data<PgPool>,
    payload: Json<CreateAuctionPayload>,
) -> Result<HttpResponse, AppError> {
    if payload.end_time <= payload.start_time {
        return Err(AppError::ValidationError("End time must be after start time.".into()));
    }
    if payload.starting_bid <= 0.0 {
        return Err(AppError::ValidationError("Starting bid must be positive.".into()));
    }

    let new_auction: Auction = sqlx::query_as_unchecked!(
        Auction,
        r#"
        INSERT INTO auctions (vehicle_id, start_time, end_time, starting_bid, status)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, vehicle_id, start_time, end_time, starting_bid,
                  current_highest_bid, highest_bidder_id, status,
                  created_at, updated_at
        "#,
        payload.vehicle_id,
        payload.start_time,
        payload.end_time,
        payload.starting_bid,
        payload.status.clone().unwrap_or_else(|| "pending".into()),
    )
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(new_auction))
}

#[put("/{auction_id}")]
pub async fn update_auction(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
    payload: Json<UpdateAuctionPayload>,
) -> Result<HttpResponse, AppError> {
    let auction_id = path.into_inner();

    if let (Some(start), Some(end)) = (payload.start_time, payload.end_time) {
        if end <= start {
            return Err(AppError::ValidationError("End time must be after start time.".into()));
        }
    }

    let updated: Auction = sqlx::query_as_unchecked!(
        Auction,
        r#"
        UPDATE auctions
        SET start_time = COALESCE($1, start_time),
            end_time   = COALESCE($2, end_time),
            starting_bid = COALESCE($3, starting_bid),
            status     = COALESCE($4, status)
        WHERE id = $5
        RETURNING id, vehicle_id, start_time, end_time, starting_bid,
                  current_highest_bid, highest_bidder_id, status,
                  created_at, updated_at
        "#,
        payload.start_time,
        payload.end_time,
        payload.starting_bid,
        payload.status,
        auction_id
    )
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/{auction_id}")]
pub async fn delete_auction(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let auction_id = path.into_inner();

    let deleted = sqlx::query_unchecked!(
        r#"
        DELETE FROM auctions
        WHERE id = $1
        "#,
        auction_id
    )
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if deleted == 0 {
        return Err(AppError::NotFound("Auction".into()));
    }

    Ok(HttpResponse::NoContent().finish())
}
