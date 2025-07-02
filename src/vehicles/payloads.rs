// src/vehicles/payloads.rs
use serde::{Deserialize};
use uuid::Uuid;
use bigdecimal::BigDecimal; // For price
use time::OffsetDateTime; // If any timestamps are in payloads

#[derive(Debug, Deserialize)]
pub struct CreateVehiclePayload {
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
    pub image_urls: Option<Vec<String>>, // Payloads often use Option for Vecs if they can be omitted
    pub features: Option<Vec<String>>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub is_featured: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVehiclePayload {
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub price: Option<BigDecimal>,
    pub mileage: Option<i32>,
    pub exterior_color: Option<String>,
    pub interior_color: Option<String>,
    pub engine: Option<String>,
    pub transmission: Option<String>,
    pub fuel_type: Option<String>,
    pub image_urls: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub is_featured: Option<bool>,
}