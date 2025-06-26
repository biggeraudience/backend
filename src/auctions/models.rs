use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Auction {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub starting_bid: f64,
    pub current_highest_bid: Option<f64>,
    pub highest_bidder_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Bid {
    pub id: Uuid,
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    pub bid_amount: f64,
    pub bid_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuctionPayload {
    pub vehicle_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub starting_bid: f64,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAuctionPayload {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub starting_bid: Option<f64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlaceBidPayload {
    pub bid_amount: f64,
}
