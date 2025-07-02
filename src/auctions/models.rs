/// src/auctions/models.rs (or handlers.rs at top)
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Auction {
    pub id:                    Uuid,
    pub vehicle_id:            Uuid,
    pub start_time:            OffsetDateTime,
    pub end_time:              OffsetDateTime,
    pub starting_bid:          f64,             // DOUBLE PRECISION
    pub current_highest_bid:   Option<f64>,     // DOUBLE PRECISION NULLABLE
    pub highest_bidder_id:     Option<Uuid>,    // UUID NULLABLE
    pub status:                String,          // TEXT NOT NULL
    pub created_at:            OffsetDateTime,  // TIMESTAMPTZ NOT NULL
    pub updated_at:            OffsetDateTime,  // TIMESTAMPTZ NOT NULL
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bid {
    pub id:         Uuid,
    pub auction_id: Uuid,
    pub bidder_id:  Uuid,
    pub bid_amount: f64,              // DOUBLE PRECISION
    pub bid_time:   OffsetDateTime,   // TIMESTAMPTZ NOT NULL
}
