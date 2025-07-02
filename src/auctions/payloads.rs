// src/auctions/payloads.rs
use serde::{Deserialize};
use uuid::Uuid;
use time::OffsetDateTime;
use bigdecimal::BigDecimal;

#[derive(Debug, Deserialize)]
pub struct CreateAuctionPayload {
    pub vehicle_id: Uuid,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub starting_bid: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAuctionPayload {
    pub current_highest_bid: Option<BigDecimal>,
    pub highest_bidder_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBidPayload {
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    pub amount: BigDecimal,
}