// src/vehicles/models.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;
// Change this line:
use sqlx_types_bigdecimal::BigDecimal; // Use the BigDecimal from sqlx-types-bigdecimal

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub price: BigDecimal,
    pub mileage: Option<i32>,
    pub exterior_color: Option<String>,
    pub interior_color: Option<String>,
    pub engine: Option<String>,
    pub transmission: Option<String>,
    pub fuel_type: Option<String>,
    pub image_urls: Vec<String>,
    pub features:   Vec<String>,
    pub description: Option<String>,
    pub status:     String,
    pub is_featured: bool,
    pub created_at:  OffsetDateTime,
    pub updated_at:  OffsetDateTime,
}