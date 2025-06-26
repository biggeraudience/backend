use aws_sdk_s3::{error::PutObjectError, Client};
use aws_sdk_s3::types::ByteStream;
use uuid::Uuid;
use crate::error::AppError;
use tracing::error;

pub async fn upload_file_to_s3(
    s3_client: &Client,
    bucket_name: &str,
    file_bytes: Vec<u8>,
    file_extension: &str,
) -> Result<String, AppError> {
    let key = format!("vehicles/{}.{}", Uuid::new_v4(), file_extension); // Unique key for S3 object

    let byte_stream = ByteStream::from(file_bytes);

    let put_object_output = s3_client
        .put_object()
        .bucket(bucket_name)
        .key(&key)
        .body(byte_stream)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to upload to S3: {:?}", e);
            AppError::FileUploadError(format!("S3 upload failed: {}", e))
        })?;

    // You might want to check put_object_output for success confirmation
    // e.g., if put_object_output.etag.is_some()

    let file_url = format!("https://{}.s3.amazonaws.com/{}", bucket_name, key);
    Ok(file_url)
}
