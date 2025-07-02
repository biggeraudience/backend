// src/vehicles/payloads.rs
use serde::{Deserialize};
use uuid::Uuid;
// Change this line:
use sqlx_types_bigdecimal::BigDecimal; // For price
use time::OffsetDateTime; // If any timestamps are in payloads
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateVehiclePayload {
    #[validate(length(min = 1))]
    pub make: String,
    #[validate(length(min = 1))]
    pub model: String,
    #[validate(range(min = 1900, max = 2100))] // Example validation
    pub year: i32,
    #[validate(range(min = 0.01))] // Price must be positive
    pub price: BigDecimal,
    #[validate(range(min = 0))]
    pub mileage: Option<i32>,
    pub exterior_color: Option<String>,
    pub interior_color: Option<String>,
    pub engine: Option<String>,
    pub transmission: Option<String>,
    pub fuel_type: Option<String>,
    pub image_urls: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
    pub description: Option<String>,
    #[validate(length(min = 1), contains = "available|sold|reserved|maintenance")] // Example status validation
    pub status: Option<String>,
    pub is_featured: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateVehiclePayload {
    #[validate(length(min = 1))]
    pub make: Option<String>,
    #[validate(length(min = 1))]
    pub model: Option<String>,
    #[validate(range(min = 1900, max = 2100))]
    pub year: Option<i32>,
    #[validate(range(min = 0.01))]
    pub price: Option<BigDecimal>,
    #[validate(range(min = 0))]
    pub mileage: Option<i32>,
    pub exterior_color: Option<String>,
    pub interior_color: Option<String>,
    pub engine: Option<String>,
    pub transmission: Option<String>,
    pub fuel_type: Option<String>,
    pub image_urls: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
    pub description: Option<String>,
    #[validate(length(min = 1), contains = "available|sold|reserved|maintenance")]
    pub status: Option<String>,
    pub is_featured: Option<bool>,
}