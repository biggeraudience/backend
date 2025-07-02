// src/auctions/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use sqlx::FromRow;
use bigdecimal::BigDecimal;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Auction {
    pub id:                    Uuid,
    pub vehicle_id:            Uuid,
    pub start_time:            OffsetDateTime,
    pub end_time:              OffsetDateTime,
    pub starting_bid:          BigDecimal,
    pub current_highest_bid:   Option<BigDecimal>,
    pub highest_bidder_id:     Option<Uuid>,
    pub status:                String,
    pub created_at:            OffsetDateTime,
    pub updated_at:            OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bid {
    pub id:         Uuid,
    pub auction_id: Uuid,
    pub bidder_id:  Uuid,
    pub bid_amount: BigDecimal,
    pub bid_time:   OffsetDateTime,
}