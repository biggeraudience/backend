/// src/vehicles/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Vehicle {
    pub id:             Uuid,
    pub make:           String,
    pub model:          String,
    pub year:           i32,
    pub price:          f64,              // DOUBLE PRECISION
    pub mileage:        Option<i32>,      // INT NULLABLE
    pub exterior_color: Option<String>,   // TEXT NULLABLE
    pub interior_color: Option<String>,   // TEXT NULLABLE
    pub engine:         Option<String>,   // TEXT NULLABLE
    pub transmission:   Option<String>,   // TEXT NULLABLE
    pub fuel_type:      Option<String>,   // TEXT NULLABLE
    #[sqlx(array)]
    pub image_urls:     Option<Vec<String>>, // TEXT[] NULLABLE
    #[sqlx(array)]
    pub features:       Option<Vec<String>>, // TEXT[] NULLABLE
    pub description:    Option<String>,      // TEXT NULLABLE
    pub status:         String,             // DEFAULT 'available', NOT NULL
    pub is_featured:    bool,               // DEFAULT false, NOT NULL
    pub created_at:     OffsetDateTime,     // TIMESTAMPTZ NOT NULL
    pub updated_at:     OffsetDateTime,     // TIMESTAMPTZ NOT NULL
}
