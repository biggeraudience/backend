use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse};
use futures_util::stream::StreamExt;
use once_cell::sync::Lazy;
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use sha2::{Digest, Sha1};
use std::env;
use chrono::Utc;

/// pulled from env once at startup
static CLOUD_NAME: Lazy<String> = Lazy::new(|| env::var("CLOUDINARY_CLOUD_NAME").unwrap());
static API_KEY:    Lazy<String> = Lazy::new(|| env::var("CLOUDINARY_API_KEY").unwrap());
static API_SECRET: Lazy<String> = Lazy::new(|| env::var("CLOUDINARY_API_SECRET").unwrap());

/// Stream an uploaded file up to Cloudinary
pub async fn handle_upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // Collect all file URLs
    let mut urls = Vec::new();

    while let Some(field) = payload.next().await {
        let mut field = field?;
        let content_disposition = field.content_disposition().unwrap();
        let filename = content_disposition.get_filename().unwrap_or("file");
        let mut bytes = web::BytesMut::new();

        // read file into memory (you might buffer or stream chunked in prod)
        while let Some(chunk) = field.next().await {
            let chunk = chunk?;
            bytes.extend_from_slice(&chunk);
        }

        // Prepare timestamp and signature
        let timestamp = Utc::now().timestamp();
        let to_sign = format!("timestamp={}", timestamp);
        let mut hasher = Sha1::new();
        hasher.update(format!("{}{}", to_sign, API_SECRET.as_str()));
        let signature = hex::encode(hasher.finalize());

        // Build multipart form for Cloudinary
        let part = Part::bytes(bytes.to_vec())
            .file_name(filename.to_string())
            .mime_str("application/octet-stream").unwrap();

        let form = Form::new()
            .text("api_key", API_KEY.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature)
            .part("file", part);

        // Send to Cloudinary
        let client = Client::new();
        let resp = client.post(&format!(
                "https://api.cloudinary.com/v1_1/{}/auto/upload",
                CLOUD_NAME.as_str()
            ))
            .multipart(form)
            .send()
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        if let Some(url) = json.get("secure_url").and_then(|u| u.as_str()) {
            urls.push(url.to_owned());
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "urls": urls })))
}
