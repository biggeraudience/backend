use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, web};
use futures_util::stream::StreamExt;
use once_cell::sync::Lazy;
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use sha2::{Digest, Sha1};
use hex;
use std::env;
use chrono::Utc;
use crate::error::AppError;

static CLOUD_NAME: Lazy<String> = Lazy::new(|| env::var("CLOUDINARY_CLOUD_NAME").unwrap());
static API_KEY:    Lazy<String> = Lazy::new(|| env::var("CLOUDINARY_API_KEY").unwrap());
static API_SECRET: Lazy<String> = Lazy::new(|| env::var("CLOUDINARY_API_SECRET").unwrap());

pub async fn upload_files_to_cloudinary(mut payload: Multipart) -> Result<Vec<String>, AppError> {
    let mut urls = Vec::new();

    while let Some(field) = payload.next().await {
        let mut f = field.map_err(AppError::MultipartError)?;
        let filename = f.content_disposition().and_then(|cd| cd.get_filename()).unwrap_or("file");
        let mut buf = web::BytesMut::new();
        while let Some(chunk) = f.next().await {
            buf.extend_from_slice(&chunk.map_err(AppError::MultipartError)?);
        }

        let ts = Utc::now().timestamp();
        let to_sign = format!("timestamp={}", ts);
        let mut hasher = Sha1::new();
        hasher.update(format!("{}{}", to_sign, API_SECRET.as_str()));
        let sig = hex::encode(hasher.finalize());

        let part = Part::bytes(buf.to_vec())
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")?;

        let form = Form::new()
            .text("api_key", API_KEY.clone())
            .text("timestamp", ts.to_string())
            .text("signature", sig)
            .part("file", part);

        let client = Client::new();
        let resp = client
            .post(&format!("https://api.cloudinary.com/v1_1/{}/auto/upload", CLOUD_NAME.as_str()))
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::FileUploadError(e.to_string()))?;

        let json: serde_json::Value = resp.json().await.map_err(AppError::SerdeError)?;
        if let Some(u) = json.get("secure_url").and_then(|v| v.as_str()) {
            urls.push(u.to_string());
        } else {
            return Err(AppError::FileUploadError("Missing secure_url".into()));
        }
    }

    Ok(urls)
}

pub async fn handle_upload(payload: Multipart) -> Result<HttpResponse, Error> {
    match upload_files_to_cloudinary(payload).await {
        Ok(urls) => Ok(HttpResponse::Ok().json(serde_json::json!({ "urls": urls }))),
        Err(e) => Err(Error::from(e)),
    }
}
