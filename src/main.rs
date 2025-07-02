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
mod utils; // Make sure this module exists and contains cloudinary

// Correct import for handle_upload from the utils/cloudinary module
use utils::cloudinary::handle_upload;

// Alias handlers for cleaner use in main.rs
use auth::handlers as auth_handlers;
use users::handlers as users_handlers;
use vehicles::handlers as vehicles_handlers;
use auctions::handlers as auctions_handlers;
use inquiries::handlers as inquiries_handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // IMPORTANT: Remove or comment out this line if you want sqlx to check against your live DB schema.
    // std::env::set_var("SQLX_OFFLINE", "true");

    // Initialize tracing subscriber for logging
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
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET missing"); // This needs to be available to middleware
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".into())
        .parse()
        .expect("PORT must be a number");

    // Build our Postgres pool & run migrations
    let pool = db::connection::get_connection_pool(&database_url)
        .await
        .expect("Failed to create DB pool");
    sqlx::migrate!("./src/db/migrations") // Ensure this path is correct relative to Cargo.toml
        .run(&pool)
        .await
        .expect("Database migrations failed");

    let bind_addr = format!("0.0.0.0:{}", port);
    tracing::info!("🚀 Starting server at http://{}", bind_addr);

    HttpServer::new(move || {
        // CORS policy
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req| {
                // In debug mode, allow all origins. In release, restrict to your domain.
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
            .app_data(web::Data::new(jwt_secret.clone())) // Pass JWT secret to app_data
            // healthcheck
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("🚀 Manga Autos API up!") }),
            )

            // --- AUTH ROUTES ---
            .service(
                web::scope("/auth")
                    .service(auth_handlers::register_user) // Renamed from 'register'
                    .service(auth_handlers::login_user),    // Renamed from 'login'
            )

            // --- PUBLIC VEHICLE ROUTES ---
            // These routes are intentionally public for displaying vehicle listings.
            .service(
                web::scope("/vehicles")
                    .service(vehicles_handlers::get_all_vehicles)
                    .service(vehicles_handlers::get_vehicle_detail)
                    .service(vehicles_handlers::get_featured_vehicles),
            )

            // --- ADMIN VEHICLE ROUTES ---
            // These routes require JWT authentication and an 'admin' role.
            .service(
                web::scope("/admin/vehicles")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(vehicles_handlers::create_vehicle)
                    .service(vehicles_handlers::update_vehicle)
                    .service(vehicles_handlers::delete_vehicle)
                    .route("/upload", web::post().to(handle_upload)), // Cloudinary upload
            )

            // --- PUBLIC AUCTION ROUTES ---
            // These GET routes are public for viewing auctions.
            .service(
                web::scope("/auctions")
                    .service(auctions_handlers::get_all_auctions)
                    .service(auctions_handlers::get_auction_detail),
            )

            // --- AUTHENTICATED BIDDING ROUTES ---
            // Placing a bid requires a logged-in user (JWT), but not necessarily admin.
            .service(
                web::scope("/auctions")
                    .wrap(auth::middleware::JwtAuth)
                    .service(auctions_handlers::create_bid), // POST /auctions/{id}/bids
            )

            // --- ADMIN AUCTION ROUTES ---
            // These routes require JWT authentication and an 'admin' role for auction management.
            .service(
                web::scope("/admin/auctions")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(auctions_handlers::create_auction)
                    .service(auctions_handlers::update_auction)
                    .service(auctions_handlers::delete_auction),
            )

            // --- PUBLIC INQUIRY ROUTES ---
            // Submitting an inquiry is public.
            .service(
                web::scope("/inquiries")
                    .service(inquiries_handlers::create_inquiry), // Renamed from 'submit_inquiry'
            )

            // --- ADMIN INQUIRY ROUTES ---
            // Managing inquiries requires JWT authentication and an 'admin' role.
            .service(
                web::scope("/admin/inquiries")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(inquiries_handlers::get_all_inquiries) // Renamed from 'list_inquiries'
                    .service(inquiries_handlers::get_inquiry_detail)
                    .service(inquiries_handlers::update_inquiry_status)
                    .service(inquiries_handlers::delete_inquiry),
            )

            // --- USER-SPECIFIC ROUTES ---
            // These routes are for a logged-in user to manage their own profile.
            .service(
                web::scope("/users")
                    .wrap(auth::middleware::JwtAuth) // Requires JWT, no admin role needed.
                    .service(users_handlers::get_me)
                    .service(users_handlers::update_me),
            )

            // --- ADMIN USER MANAGEMENT ROUTES ---
            // These routes are for administrators to manage all user accounts.
            .service(
                web::scope("/admin/users")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(users_handlers::list_users)
                    .service(users_handlers::get_user_by_id)
                    .service(users_handlers::update_user_role),
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