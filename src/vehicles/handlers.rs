use actix_web::{web, HttpResponse, post, delete, put, get};
use sqlx::{PgPool, FromRow, Postgres};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

use crate::error::AppError;

// --- Struct Definitions ---
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub price: f64, // Use f64 for DECIMAL(12,2) in Rust, requires `bigdecimal` feature if you want `BigDecimal` type
    pub mileage: Option<i32>,
    pub exterior_color: Option<String>,
    pub interior_color: Option<String>,
    pub engine: Option<String>,
    pub transmission: Option<String>,
    pub fuel_type: Option<String>,
    #[sqlx(json)]
    pub image_urls: Vec<String>,
    #[sqlx(json)]
    pub features: Vec<String>,
    pub description: Option<String>,
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
    pub mileage: Option<i32>,
    pub exterior_color: Option<String>,
    pub interior_color: Option<String>,
    pub engine: Option<String>,
    pub transmission: Option<String>,
    pub fuel_type: Option<String>,
    pub image_urls: Vec<String>,
    pub features: Vec<String>,
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

// --- Handler Functions ---

/// Handles creating a new vehicle.
#[post("/vehicles")]
pub async fn create_vehicle(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateVehiclePayload>,
) -> Result<HttpResponse, AppError> {
    let now = OffsetDateTime::now_utc();
    let status = payload.status.as_deref().unwrap_or("available");
    let is_featured = payload.is_featured.unwrap_or(false);

    let new_vehicle = sqlx::query_as!(
        Vehicle,
        r#"
        INSERT INTO vehicles (
            make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
        )
        RETURNING id, make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at
        "#,
        payload.make,
        payload.model,
        payload.year,
        payload.price,
        payload.mileage,
        payload.exterior_color,
        payload.interior_color,
        payload.engine,
        payload.transmission,
        payload.fuel_type,
        &payload.image_urls, // Borrow here
        &payload.features,   // Borrow here
        payload.description,
        status,
        is_featured,
        now,
        now
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Created().json(new_vehicle))
}

/// Handles updating an existing vehicle.
#[put("/vehicles/{id}")]
pub async fn update_vehicle(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    payload: web::Json<UpdateVehiclePayload>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let now = OffsetDateTime::now_utc();

    let mut query_builder: sqlx::QueryBuilder<Postgres> =
        sqlx::QueryBuilder::new("UPDATE vehicles SET updated_at = ");
    query_builder.push_bind(now);

    // Conditionally add fields to update, borrowing from payload
    if let Some(make) = &payload.make { // Borrow
        query_builder.push(", make = ");
        query_builder.push_bind(make);
    }
    if let Some(model) = &payload.model { // Borrow
        query_builder.push(", model = ");
        query_builder.push_bind(model);
    }
    if let Some(year) = payload.year {
        query_builder.push(", year = ");
        query_builder.push_bind(year);
    }
    if let Some(price) = payload.price {
        query_builder.push(", price = ");
        query_builder.push_bind(price);
    }
    if let Some(mileage) = payload.mileage {
        query_builder.push(", mileage = ");
        query_builder.push_bind(mileage);
    }
    if let Some(exterior_color) = &payload.exterior_color { // Borrow
        query_builder.push(", exterior_color = ");
        query_builder.push_bind(exterior_color);
    }
    if let Some(interior_color) = &payload.interior_color { // Borrow
        query_builder.push(", interior_color = ");
        query_builder.push_bind(interior_color);
    }
    if let Some(engine) = &payload.engine { // Borrow
        query_builder.push(", engine = ");
        query_builder.push_bind(engine);
    }
    if let Some(transmission) = &payload.transmission { // Borrow
        query_builder.push(", transmission = ");
        query_builder.push_bind(transmission);
    }
    if let Some(fuel_type) = &payload.fuel_type { // Borrow
        query_builder.push(", fuel_type = ");
        query_builder.push_bind(fuel_type);
    }
    if let Some(image_urls) = &payload.image_urls { // Already borrowing from before
        query_builder.push(", image_urls = ");
        query_builder.push_bind(image_urls);
    }
    if let Some(features) = &payload.features { // Already borrowing from before
        query_builder.push(", features = ");
        query_builder.push_bind(features);
    }
    if let Some(description) = &payload.description { // Borrow
        query_builder.push(", description = ");
        query_builder.push_bind(description);
    }
    if let Some(status) = &payload.status { // Borrow
        query_builder.push(", status = ");
        query_builder.push_bind(status);
    }
    if let Some(is_featured) = payload.is_featured {
        query_builder.push(", is_featured = ");
        query_builder.push_bind(is_featured);
    }

    query_builder.push(" WHERE id = ");
    query_builder.push_bind(id);
    query_builder.push(" RETURNING id, make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at");


    let updated_vehicle = query_builder
        .build_query_as::<Vehicle>()
        .fetch_one(&**pool)
        .await?;

    Ok(HttpResponse::Ok().json(updated_vehicle))
}

/// Handles deleting a vehicle.
#[delete("/vehicles/{id}")]
pub async fn delete_vehicle(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let deleted_rows = sqlx::query!("DELETE FROM vehicles WHERE id = $1", id)
        .execute(&**pool)
        .await?
        .rows_affected();

    if deleted_rows == 0 {
        return Err(AppError::NotFound(format!("Vehicle with id {} not found", id)));
    }

    Ok(HttpResponse::NoContent().finish())
}

/// Handles fetching all vehicles.
#[get("/vehicles")]
pub async fn get_all_vehicles(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let vehicles = sqlx::query_as!(
        Vehicle,
        r#"SELECT id, make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at FROM vehicles ORDER BY created_at DESC"#
    )
    .fetch_all(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(vehicles))
}

/// Handles fetching a single vehicle by ID.
#[get("/vehicles/{id}")]
pub async fn get_vehicle_detail(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let vehicle = sqlx::query_as!(
        Vehicle,
        r#"SELECT id, make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at FROM vehicles WHERE id = $1"#,
        id
    )
    .fetch_optional(&**pool)
    .await?;

    match vehicle {
        Some(v) => Ok(HttpResponse::Ok().json(v)),
        None => Err(AppError::NotFound(format!("Vehicle with id {} not found", id))),
    }
}

/// Handles fetching featured vehicles.
#[get("/vehicles/featured")]
pub async fn get_featured_vehicles(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let vehicles = sqlx::query_as!(
        Vehicle,
        r#"SELECT id, make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at FROM vehicles WHERE is_featured = TRUE ORDER BY created_at DESC"#
    )
    .fetch_all(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(vehicles))
}