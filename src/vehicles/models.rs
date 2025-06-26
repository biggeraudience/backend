use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: Option<String>,
    pub vin: String,
    pub price: f64, // Use f64 for DECIMAL from DB, or Decimal crate
    pub mileage: i32,
    pub description: Option<String>,
    pub image_urls: Vec<String>, // Array of image URLs
    pub status: String,
    pub is_featured: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVehiclePayload {
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: Option<String>,
    pub vin: String,
    pub price: f64,
    pub mileage: i32,
    pub description: Option<String>,
    pub status: Option<String>,
    pub is_featured: Option<bool>,
    // Image URLs will come from file uploads, not directly in this payload.
}

#[derive(Debug, Deserialize)]
pub struct UpdateVehiclePayload {
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub vin: Option<String>,
    pub price: Option<f64>,
    pub mileage: Option<i32>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub is_featured: Option<bool>,
    // Image URLs to add/remove might be handled in a separate endpoint or complex payload
}

#[derive(Debug, Deserialize)]
pub struct VehicleQueryParams {
    pub make: Option<String>,
    pub model: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
