// src/vehicles/handlers.rs
use actix_web::{get, post, put, delete, web, HttpResponse};
use sqlx::{PgPool, Postgres};
use sqlx::QueryBuilder;
use uuid::Uuid;
use time::OffsetDateTime;
// Change this line:
use sqlx_types_bigdecimal::BigDecimal; // Use the BigDecimal from sqlx-types-bigdecimal

use crate::error::AppError;
use crate::vehicles::models::Vehicle; // Import Vehicle from models
use crate::vehicles::payloads::{CreateVehiclePayload, UpdateVehiclePayload}; // Import payloads from payloads.rs


/// Create a new vehicle (admin only)
#[post("/vehicles")]
pub async fn create_vehicle(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateVehiclePayload>,
) -> Result<HttpResponse, AppError> {
    // Validate the payload using the `validator` crate
    payload.validate()?;

    let now = OffsetDateTime::now_utc();
    // Default values if not provided in payload (can also be handled via Default trait on payload)
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
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11, $12, $13,
            $14, $15, $16, $17
        )
        RETURNING
            id, make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at
        "#,
        &payload.make,
        &payload.model,
        payload.year,
        payload.price,
        payload.mileage,
        &payload.exterior_color,
        &payload.interior_color,
        &payload.engine,
        &payload.transmission,
        &payload.fuel_type,
        payload.image_urls.as_ref().unwrap_or(&vec![]), // Handles Option<Vec<String>>
        payload.features.as_ref().unwrap_or(&vec![]),   // Handles Option<Vec<String>>
        &payload.description, // FIXED: Missing description binding ($13 for SQL, was missing here)
        status,
        is_featured,
        now,
        now
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Created().json(new_vehicle))
}

/// Update an existing vehicle (admin only)
#[put("/vehicles/{id}")]
pub async fn update_vehicle(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    payload: web::Json<UpdateVehiclePayload>,
) -> Result<HttpResponse, AppError> {
    // Validate the payload using the `validator` crate
    payload.validate()?;

    let id = path.into_inner();
    let now = OffsetDateTime::now_utc();

    // Build dynamic UPDATE
    let mut qb: QueryBuilder<Postgres> =
        sqlx::QueryBuilder::new("UPDATE vehicles SET updated_at = ");
    qb.push_bind(now);

    if let Some(make) = &payload.make {
        qb.push(", make = ").push_bind(make);
    }
    if let Some(model) = &payload.model {
        qb.push(", model = ").push_bind(model);
    }
    if let Some(year) = payload.year {
        qb.push(", year = ").push_bind(year);
    }
    if let Some(price) = payload.price {
        // Price validation is now handled by `validator` crate on the payload
        qb.push(", price = ").push_bind(price);
    }
    if let Some(mileage) = payload.mileage {
        // Mileage validation is now handled by `validator` crate on the payload
        qb.push(", mileage = ").push_bind(mileage);
    }
    if let Some(color) = &payload.exterior_color {
        qb.push(", exterior_color = ").push_bind(color);
    }
    if let Some(color) = &payload.interior_color {
        qb.push(", interior_color = ").push_bind(color);
    }
    if let Some(engine) = &payload.engine {
        qb.push(", engine = ").push_bind(engine);
    }
    if let Some(transmission) = &payload.transmission {
        qb.push(", transmission = ").push_bind(transmission);
    }
    if let Some(fuel) = &payload.fuel_type {
        qb.push(", fuel_type = ").push_bind(fuel);
    }
    if let Some(urls) = &payload.image_urls {
        qb.push(", image_urls = ").push_bind(urls);
    }
    if let Some(features) = &payload.features {
        qb.push(", features = ").push_bind(features);
    }
    if let Some(desc) = &payload.description {
        qb.push(", description = ").push_bind(desc);
    }
    if let Some(status) = &payload.status {
        // Status validation is now handled by `validator` crate on the payload
        qb.push(", status = ").push_bind(status);
    }
    if let Some(feat) = payload.is_featured {
        qb.push(", is_featured = ").push_bind(feat);
    }

    qb.push(" WHERE id = ").push_bind(id);
    qb.push(" RETURNING
        id, make, model, year, price, mileage, exterior_color, interior_color,
        engine, transmission, fuel_type, image_urls, features, description,
        status, is_featured, created_at, updated_at
    ");

    let updated = qb
        .build_query_as::<Vehicle>()
        .fetch_one(&**pool)
        .await?;

    Ok(HttpResponse::Ok().json(updated))
}

/// Delete a vehicle (admin only)
#[delete("/vehicles/{id}")]
pub async fn delete_vehicle(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let rows = sqlx::query!("DELETE FROM vehicles WHERE id = $1", id)
        .execute(&**pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("Vehicle {} not found", id)));
    }
    Ok(HttpResponse::NoContent().finish())
}

/// Get all vehicles (public)
#[get("/vehicles")]
pub async fn get_all_vehicles(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let vehicles = sqlx::query_as!(
        Vehicle,
        r#"
        SELECT
            id, make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at
        FROM vehicles
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(vehicles))
}

/// Get one vehicle by ID (public)
#[get("/vehicles/{id}")]
pub async fn get_vehicle_detail(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let opt = sqlx::query_as!(
        Vehicle,
        r#"
        SELECT
            id, make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at
        FROM vehicles
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&**pool)
    .await?;

    match opt {
        Some(v) => Ok(HttpResponse::Ok().json(v)),
        None    => Err(AppError::NotFound(format!("Vehicle {} not found", id))),
    }
}

/// Get featured vehicles (public)
#[get("/vehicles/featured")]
pub async fn get_featured_vehicles(
    pool: web::Data<PgPool>
) -> Result<HttpResponse, AppError> {
    let vehicles = sqlx::query_as!(
        Vehicle,
        r#"
        SELECT
            id, make, model, year, price, mileage, exterior_color, interior_color,
            engine, transmission, fuel_type, image_urls, features, description,
            status, is_featured, created_at, updated_at
        FROM vehicles
        WHERE is_featured = TRUE
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(vehicles))
}