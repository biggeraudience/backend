use serde::Deserialize;
use envy;
\ n#[derive(Deserialize)]
pub struct Settings { pub database_url: String, pub jwt_secret: String, pub port: String, pub cloudinary_cloud_name: String, pub cloudinary_api_key: String, pub cloudinary_api_secret: String, pub cloudinary_upload_preset: String, }

lazy_static! { pub static ref SETTINGS: Settings = envy::from_env::<Settings>().expect("Invalid ENV"); }

pub fn get() -> &'static Settings { &SETTINGS }
