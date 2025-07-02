use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub price: f64,
    pub mileage: i32,
    pub exterior_color: String,
    pub interior_color: String,
    pub engine: String,
    pub transmission: String,
    pub fuel_type: String,
    pub image_urls: Vec<String>, // Assuming NOT NULL, if nullable, change to Option<Vec<String>>
    pub features: Vec<String>,   // Assuming NOT NULL, if nullable, change to Option<Vec<String>>
    pub description: Option<String>, // Description is usually optional
    pub status: String,
    pub is_featured: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateVehiclePayload {
    pub make: String,
    pub model: String,
    pub year: i32,
    pub price: f64,
    pub mileage: i32,
    pub exterior_color: String,
    pub interior_color: String,
    pub engine: String,
    pub transmission: String,
    pub fuel_type: String,
    pub image_urls: Option<Vec<String>>, // Made Option for flexibility in creation
    pub features: Option<Vec<String>>,   // Made Option for flexibility in creation
    pub description: Option<String>,
    pub status: Option<String>,
    pub is_featured: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVehiclePayload {
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub price: Option<f64>,
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