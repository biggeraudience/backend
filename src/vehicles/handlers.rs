// src/vehicles/handlers.rs
use actix_web::{get, post, put, delete, web, HttpResponse};
use sqlx::PgPool;
use web::{Data, Json};
use uuid::Uuid;
use chrono::Utc; // Used for timestamps like updated_at

use crate::error::AppError;
use crate::vehicles::models::{Vehicle, CreateVehiclePayload, UpdateVehiclePayload};

#[get("/")]
pub async fn get_all_vehicles(pool: Data<PgPool>) -> Result<HttpResponse, AppError> {
    // Explicitly specify Vec<Vehicle> as the return type
    let list: Vec<Vehicle> = sqlx::query_as!(Vehicle, r#"
        SELECT
            id, make, model, year, price, mileage, exterior_color, interior_color, engine,
            transmission, fuel_type, image_urls, features, description, status, is_featured,
            created_at, updated_at
        FROM vehicles
        ORDER BY created_at DESC
        "#)
        .fetch_all(pool.get_ref())
        .await?;
    Ok(HttpResponse::Ok().json(list))
}

#[get("/{vehicle_id}")]
pub async fn get_vehicle_detail(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let vehicle_id = path.into_inner();
    // Explicitly specify Vehicle as the return type
    let vehicle: Vehicle = sqlx::query_as!(
        Vehicle,
        r#"
        SELECT
            id, make, model, year, price, mileage, exterior_color, interior_color, engine,
            transmission, fuel_type, image_urls, features, description, status, is_featured,
            created_at, updated_at
        FROM vehicles
        WHERE id = $1
        "#,
        vehicle_id
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Vehicle".to_string()))?;

    Ok(HttpResponse::Ok().json(vehicle))
}

#[get("/featured")]
pub async fn get_featured_vehicles(pool: Data<PgPool>) -> Result<HttpResponse, AppError> {
    // Explicitly specify Vec<Vehicle> as the return type
    let featured_vehicles: Vec<Vehicle> = sqlx::query_as!(
        Vehicle,
        r#"
        SELECT
            id, make, model, year, price, mileage, exterior_color, interior_color, engine,
            transmission, fuel_type, image_urls, features, description, status, is_featured,
            created_at, updated_at
        FROM vehicles
        WHERE is_featured = TRUE AND status = 'available'
        ORDER BY created_at DESC
        LIMIT 10 -- Example limit
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(featured_vehicles))
}


#[post("/")]
pub async fn create_vehicle(
    pool: Data<PgPool>,
    payload: Json<CreateVehiclePayload>,
) -> Result<HttpResponse, AppError> {
    // Basic validation (add more as needed)
    if payload.make.is_empty() || payload.model.is_empty() || payload.year == 0 || payload.price == 0.0 {
        return Err(AppError::ValidationError("Make, model, year, and price are required.".to_string()));
    }

    // Explicitly specify Vehicle as the return type
    let new_vehicle: Vehicle = sqlx::query_as!(
        Vehicle,
        r#"
        INSERT INTO vehicles (
            make, model, year, price, mileage, exterior_color, interior_color, engine,
            transmission, fuel_type, image_urls, features, description, status, is_featured
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING
            id, make, model, year, price, mileage, exterior_color, interior_color, engine,
            transmission, fuel_type, image_urls, features, description, status, is_featured,
            created_at, updated_at
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
        &payload.image_urls, // Bind as array
        &payload.features,   // Bind as array
        payload.description,
        payload.status.clone().unwrap_or_else(|| "available".to_string()),
        payload.is_featured.unwrap_or(false)
    )
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(new_vehicle))
}

#[put("/{vehicle_id}")]
pub async fn update_vehicle(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
    payload: Json<UpdateVehiclePayload>,
) -> Result<HttpResponse, AppError> {
    let vehicle_id = path.into_inner();

    // Explicitly specify Vehicle as the return type
    let updated_vehicle: Vehicle = sqlx::query_as!(
        Vehicle,
        r#"
        UPDATE vehicles
        SET
            make = COALESCE($1, make),
            model = COALESCE($2, model),
            year = COALESCE($3, year),
            price = COALESCE($4, price),
            mileage = COALESCE($5, mileage),
            exterior_color = COALESCE($6, exterior_color),
            interior_color = COALESCE($7, interior_color),
            engine = COALESCE($8, engine),
            transmission = COALESCE($9, transmission),
            fuel_type = COALESCE($10, fuel_type),
            image_urls = COALESCE($11, image_urls),
            features = COALESCE($12, features),
            description = COALESCE($13, description),
            status = COALESCE($14, status),
            is_featured = COALESCE($15, is_featured),
            updated_at = $16
        WHERE id = $17
        RETURNING
            id, make, model, year, price, mileage, exterior_color, interior_color, engine,
            transmission, fuel_type, image_urls, features, description, status, is_featured,
            created_at, updated_at
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
        // For array types, you usually need to pass a reference to a slice or Vec
        payload.image_urls.as_ref().map(|v| v as _), // Cast to &[_] or &Vec<String>
        payload.features.as_ref().map(|v| v as _),
        payload.description,
        payload.status,
        payload.is_featured,
        Utc::now(), // Set updated_at to current time
        vehicle_id
    )
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(updated_vehicle))
}

#[delete("/{vehicle_id}")]
pub async fn delete_vehicle(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let vehicle_id = path.into_inner();

    let deleted_rows = sqlx::query!(
        r#"
        DELETE FROM vehicles
        WHERE id = $1
        "#,
        vehicle_id
    )
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if deleted_rows == 0 {
        return Err(AppError::NotFound("Vehicle".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}