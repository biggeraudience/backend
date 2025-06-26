// src/vehicles/utils.rs
use actix_multipart::Multipart;
use crate::error::AppError;
use crate::utils::cloudinary::upload_files_to_cloudinary; // Import the reusable Cloudinary function

// This function acts as the utility for vehicle-related image uploads.
// It uses the generic Cloudinary upload function.
pub async fn upload_vehicle_images(
    payload: Multipart,
) -> Result<Vec<String>, AppError> {
    // Simply call the core Cloudinary upload logic
    upload_files_to_cloudinary(payload).await
}