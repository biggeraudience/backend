// src/auctions/payloads.rs
use serde::{Deserialize};
use uuid::Uuid;
use time::OffsetDateTime;
// Change this line:
use sqlx_types_bigdecimal::BigDecimal; // Use the BigDecimal from sqlx-types-bigdecimal
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAuctionPayload {
    pub vehicle_id: Uuid,
    #[validate(custom = "validate_start_end_time")]
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    #[validate(range(min = 0.01))] // Example: starting bid must be positive
    pub starting_bid: BigDecimal,
}

// Custom validation function example (can be in the same file or a utils file)
fn validate_start_end_time(start: &OffsetDateTime, end: &OffsetDateTime) -> Result<(), validator::ValidationError> {
    if start >= end {
        return Err(validator::ValidationError::new("start_time_after_end_time")
            .with_message("Start time must be before end time".into())
            .with_code("invalid_time_range".into()));
    }
    // You might also add checks like start_time not in the past etc.
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAuctionPayload {
    pub current_highest_bid: Option<BigDecimal>,
    pub highest_bidder_id: Option<Uuid>,
    #[validate(length(min = 1), contains = "active|completed|cancelled")] // Example status validation
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBidPayload {
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    #[validate(range(min = 0.01))] // Bid amount must be positive
    pub amount: BigDecimal,
}