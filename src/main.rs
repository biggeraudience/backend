use actix_web::{get, App, HttpServer, Responder};
use std::env; // Import the env module to read environment variables

#[get("/")]
async fn health() -> impl Responder {
    "Manga Automobiles API up and running!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Read the PORT environment variable provided by Render.
    // Default to "8000" if not set (useful for local development).
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .expect("PORT must be a number");

    let host = "0.0.0.0"; // Bind to all network interfaces
    let address = format!("{}:{}", host, port);

    println!("Starting backend on {}", address);
    HttpServer::new(|| App::new().service(health))
        .bind(&address)? // Bind to the dynamic address
        .run()
        .await
}