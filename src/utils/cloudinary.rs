use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use crate::error::AppError;
use futures_util::TryStreamExt as _;
use std::env;
use sha1::{Sha1, Digest};
use hex;

/// Uploads each field in the multipart payload to Cloudinary
/// and returns a Vec of secure URLs.
pub async fn upload_files_to_cloudinary(mut payload: Multipart) -> Result<Vec<String>, AppError> {
    let mut urls = Vec::new();

    let cloud_name = env::var("CLOUDINARY_CLOUD_NAME")
        .unwrap_or_else(|_| "your_cloud_name".to_string());

    while let Some(mut field) = payload.try_next().await? {
        let mut bytes = web::BytesMut::new();
        while let Some(chunk) = field.try_next().await? {
            bytes.extend_from_slice(&chunk);
        }
        let data = bytes.freeze();

        // Generate a simple SHA-1 based identifier for the upload
        let mut hasher = Sha1::new();
        hasher.update(&data);
        let signature = hex::encode(hasher.finalize());

        // Stub URL format; replace with real Cloudinary SDK call in production
        let fake_url = format!(
            "https://res.cloudinary.com/{}/image/upload/v[mock]/{}.jpg",
            cloud_name,
            signature
        );
        urls.push(fake_url);
    }

    Ok(urls)
}

/// Actix handler for `/admin/vehicles/upload`
pub async fn handle_upload(
    payload: Multipart,
) -> Result<HttpResponse, AppError> {
    let urls = upload_files_to_cloudinary(payload).await?;
    Ok(HttpResponse::Ok().json(urls))
}