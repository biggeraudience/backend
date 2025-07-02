use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Auction {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub starting_bid: f64,
    pub current_highest_bid: Option<f64>, // Made Option<f64>
    pub highest_bidder_id: Option<Uuid>,  // Made Option<Uuid>
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, FromRow)]
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
    pub status: Option<String>, // Optional status during creation
}

#[derive(Debug, Deserialize)]
pub struct UpdateAuctionPayload {
    pub start_time: Option<OffsetDateTime>,
    pub end_time: Option<OffsetDateTime>,
    pub starting_bid: Option<f64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlaceBidPayload {
    pub bid_amount: f64,
}