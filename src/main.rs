use actix_web::{web, App, HttpServer, HttpResponse};
use actix_cors::Cors;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod error;
mod auth;
mod users;
mod vehicles;
mod auctions;
mod inquiries;
mod utils;

use utils::cloudinary::handle_upload;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  
    std::env::set_var("SQLX_OFFLINE", "true");

   
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::new(
                env::var("RUST_LOG")
                    .unwrap_or_else(|_| "info,backend=debug,sqlx=info".into()),
            ),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load .env (if present) and then our required env vars
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let jwt_secret   = env::var("JWT_SECRET").expect("JWT_SECRET missing");
    let port: u16    = env::var("PORT")
        .unwrap_or_else(|_| "8000".into())
        .parse()
        .expect("PORT must be a number");

    // Build our Postgres pool & run migrations
    let pool = db::connection::get_connection_pool(&database_url)
        .await
        .expect("Failed to create DB pool");
    sqlx::migrate!("./src/db/migrations")
        .run(&pool)
        .await
        .expect("Database migrations failed");

    let bind_addr = format!("0.0.0.0:{}", port);
    tracing::info!("🚀 Starting server at http://{}", bind_addr);

    HttpServer::new(move || {
        // CORS policy
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req| {
                cfg!(debug_assertions)
                    || origin.as_bytes().ends_with(b"mangaautomobiles.com")
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allow_any_header()
            .supports_credentials();

        App::new()
            // middleware
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            // shared app state
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            // healthcheck
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("🚀 Manga Autos API up!") }),
            )

            // ─── AUTH ─────────────────────────────────────────
            .service(
                web::scope("/auth")
                    .service(auth::handlers::register)
                    .service(auth::handlers::login),
            )

            // ─── VEHICLES ─────────────────────────────────────
            .service(
                web::scope("/vehicles")
                    .service(vehicles::handlers::get_all_vehicles)
                    .service(vehicles::handlers::get_vehicle_detail)
                    .service(vehicles::handlers::get_featured_vehicles),
            )
            .service(
                web::scope("/admin/vehicles")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(vehicles::handlers::create_vehicle)
                    .service(vehicles::handlers::update_vehicle)
                    .service(vehicles::handlers::delete_vehicle)
                    .route("/upload", web::post().to(handle_upload)),
            )

            // ─── AUCTIONS ─────────────────────────────────────
            .service(
                web::scope("/auctions")
                    .service(auctions::handlers::get_all_auctions)
                    .service(auctions::handlers::get_auction_detail)
                    .wrap(auth::middleware::JwtAuth)
                    .service(auctions::handlers::place_bid),
            )
            .service(
                web::scope("/admin/auctions")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(auctions::handlers::create_auction)
                    .service(auctions::handlers::update_auction)
                    .service(auctions::handlers::delete_auction),
            )

            // ─── INQUIRIES ────────────────────────────────────
            .service(
                web::scope("/inquiries")
                    .service(inquiries::handlers::submit_inquiry),
            )
            .service(
                web::scope("/admin/inquiries")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(inquiries::handlers::list_inquiries)
                    .service(inquiries::handlers::update_inquiry_status)
                    .service(inquiries::handlers::delete_inquiry),
            )

            // ─── USERS ────────────────────────────────────────
            .service(
                web::scope("/users")
                    .wrap(auth::middleware::JwtAuth)
                    .service(users::handlers::get_me)
                    .service(users::handlers::update_me),
            )
            .service(
                web::scope("/admin/users")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(users::handlers::list_users)
                    .service(users::handlers::get_user_by_id)
                    .service(users::handlers::update_user_role),
            )

            // catch 404
            .default_service(web::to(|| async {
                HttpResponse::NotFound().body("404 Not Found")
            }))
    })
    .bind(bind_addr)?
    .run()
    .await
}
