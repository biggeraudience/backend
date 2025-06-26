// src/main.rs
use actix_web::{web, App, HttpServer, HttpResponse};
use actix_cors::Cors;
// Removed: use sqlx::PgPool; // Not directly used in main.rs functions

use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// your modules
mod db;         // Assumes src/db/mod.rs exists and contains `pub mod connection;`
mod error;      // error.rs
mod auth;       // auth/{handlers,models,utils,middleware}.rs
mod users;      // users/{handlers,models}.rs
mod vehicles;   // vehicles/{handlers,models}.rs
mod auctions;   // auctions/{handlers,models}.rs
mod inquiries;  // inquiries/{handlers,models}.rs
mod utils;      // Assumes src/utils/mod.rs exists and contains `pub mod cloudinary;`

// bring in the Cloudinary upload handler
use utils::cloudinary::handle_upload; // Correct import path based on module structure

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1) Structured logging via tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,backend=debug,sqlx=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    dotenvy::dotenv().ok();

    // 2) Load core env vars
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let jwt_secret   = env::var("JWT_SECRET").expect("JWT_SECRET missing");
    let port: u16    = env::var("PORT").unwrap_or_else(|_| "8000".into())
                            .parse()
                            .expect("PORT must be a number");

    // 3) Connect to Postgres and run migrations
    let pool = db::connection::get_connection_pool(&database_url)
        .await
        .expect("Failed to create DB pool");

    // Retained the original path as per user input, ensure migration file follows `0001_name.sql` format
    sqlx::migrate!("./src/db/migrations")
        .run(&pool)
        .await
        .expect("Database migrations failed");

    let bind_addr = format!("0.0.0.0:{}", port);
    tracing::info!("Listening on http://{}\n", bind_addr);

    // 4) Start the HTTP server
    HttpServer::new(move || {
        // CORS policy: allow your frontend in prod, or all in debug
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req| {
                cfg!(debug_assertions)
                    || origin.as_bytes().ends_with(b"mangaautomobiles.com")
            })
            .allowed_methods(vec!["GET","POST","PUT","DELETE"])
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())

            // make Postgres pool & JWT secret available to all handlers
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))

            // healthcheck
            .route("/", web::get().to(|| async {
                HttpResponse::Ok().body("🚀 Manga Autos API up!")
            }))

            // ─── AUTH ───────────────────────────────────────────────────────────────
            .service(
                web::scope("/auth")
                    .service(auth::handlers::register)
                    .service(auth::handlers::login)
                // + forgot/reset when ready
            )

            // ─── VEHICLES ───────────────────────────────────────────────────────────
            .service(
                web::scope("/vehicles")
                    .service(vehicles::handlers::get_all_vehicles)
                    .service(vehicles::handlers::get_vehicle_detail)
                    .service(vehicles::handlers::get_featured_vehicles)
            )
            // admin CRUD + image‐upload
            .service(
                web::scope("/admin/vehicles")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(vehicles::handlers::create_vehicle)
                    .service(vehicles::handlers::update_vehicle)
                    .service(vehicles::handlers::delete_vehicle)
                    // ← new Cloudinary upload endpoint
                    .route("/upload", web::post().to(handle_upload))
            )

            // ─── AUCTIONS ──────────────────────────────────────────────────────────
            .service(
                web::scope("/auctions")
                    .service(auctions::handlers::get_all_auctions)
                    .service(auctions::handlers::get_auction_detail)
                    // place bid must be authenticated
                    .service(
                        web::scope("")
                            .wrap(auth::middleware::JwtAuth)
                            .service(auctions::handlers::place_bid)
                    )
            )
            .service(
                web::scope("/admin/auctions")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(auctions::handlers::create_auction)
                    .service(auctions::handlers::update_auction)
                    .service(auctions::handlers::delete_auction)
            )

            // ─── INQUIRIES ─────────────────────────────────────────────────────────
            .service(
                web::scope("/inquiries")
                    .service(inquiries::handlers::submit_inquiry)
            )
            .service(
                web::scope("/admin/inquiries")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(inquiries::handlers::list_inquiries)
                    .service(inquiries::handlers::update_inquiry_status)
                    .service(inquiries::handlers::delete_inquiry)
            )

            // ─── USERS ─────────────────────────────────────────────────────────────
            .service(
                web::scope("/users")
                    .wrap(auth::middleware::JwtAuth)
                    .service(users::handlers::get_me)
                    .service(users::handlers::update_me)
            )
            .service(
                web::scope("/admin/users")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(users::handlers::list_users)
                    .service(users::handlers::get_user_by_id)
                    .service(users::handlers::update_user_role)
            )

            // 404 for everything else
            .default_service(web::to(|| async {
                HttpResponse::NotFound().body("404 Not Found")
            }))
    })
    .bind(bind_addr)?
    .run()
    .await
}