use serde::{Serialize, Deserialize};
use crate::config;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};
use thiserror::Error;
use chrono::{Utc, Duration};

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: i64,
}

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("invalid token format")]
    InvalidFormat,
    #[error("base64 decoding failed")]
    Base64Decode,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("json parse error")]
    JsonError,
    #[error("token expired")]
    Expired,
}

/// Create a HS256‐signed JWT.
/// Expires in 1 hour.
pub fn create_token(user_id: &str, role: &str) -> String {
    let header = serde_json::json!({ "alg": "HS256", "typ": "JWT" });
    let exp = (Utc::now() + Duration::hours(1)).timestamp();
    let claims = Claims {
        sub: user_id.into(),
        role: role.into(),
        exp,
    };

    let header_b64 = general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
    let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_string(&claims).unwrap());
    let signing_input = format!("{}.{}", header_b64, payload_b64);

    let mut mac = HmacSha256::new_from_slice(config::get().jwt_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(signing_input.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_b64 = general_purpose::URL_SAFE_NO_PAD.encode(sig);

    format!("{}.{}.{}", header_b64, payload_b64, sig_b64)
}

/// Verify and decode a HS256‐signed JWT.
pub fn verify_token(token: &str) -> Result<Claims, JwtError> {
    let parts: Vec<_> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(JwtError::InvalidFormat);
    }
    let (header_b64, payload_b64, sig_b64) = (parts[0], parts[1], parts[2]);

    let signing_input = format!("{}.{}", header_b64, payload_b64);
    let sig = general_purpose::URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| JwtError::Base64Decode)?;

    let mut mac = HmacSha256::new_from_slice(config::get().jwt_secret.as_bytes())
        .map_err(|_| JwtError::InvalidSignature)?;
    mac.update(signing_input.as_bytes());
    mac.verify_slice(&sig).map_err(|_| JwtError::InvalidSignature)?;

    let payload_json = general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| JwtError::Base64Decode)?;
    let claims: Claims = serde_json::from_slice(&payload_json)
        .map_err(|_| JwtError::JsonError)?;

    if Utc::now().timestamp() > claims.exp {
        return Err(JwtError::Expired);
    }
    Ok(claims)
}
