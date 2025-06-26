use actix_web::{web, App, HttpServer, HttpResponse};
use actix_cors::Cors;
use sqlx::PgPool;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;         // connection.rs
mod error;      // error.rs
mod auth;       // auth/{handlers,models,utils,middleware}.rs
mod users;      // users/{handlers,models}.rs
mod vehicles;   // vehicles/{handlers,models}.rs
mod auctions;   // auctions/{handlers,models}.rs
mod inquiries;  // inquiries/{handlers,models}.rs
mod utils;      // utils/cloudinary.rs

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info,backend=debug,sqlx=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET missing");
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "8000".into()).parse().unwrap();

    let pool = db::connection::get_connection_pool(&database_url)
        .await
        .expect("DB pool");

    // Run migrations
    sqlx::migrate!("./src/db/migrations")
        .run(&pool)
        .await
        .expect("migrations");

    // AWS S3 client
    let aws_config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);
    let s3_bucket = env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME missing");

    let bind_addr = format!("0.0.0.0:{}", port);
    tracing::info!("Listening on {}", bind_addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|orig, _| cfg!(debug_assertions) || orig.as_bytes().ends_with(b"mangaautomobiles.com"))
            .allowed_methods(vec!["GET","POST","PUT","DELETE"])
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            .app_data(web::Data::new(s3_client.clone()))
            .app_data(web::Data::new(s3_bucket.clone()))

            // Healthcheck
            .route("/", web::get().to(|| async { HttpResponse::Ok().body("🚀 API up!") }))

            // Auth
            .service(
                web::scope("/auth")
                    .service(auth::handlers::register)
                    .service(auth::handlers::login)
                    // + reset/password when ready
            )
            // Vehicles (public + admin)
            .service(
                web::scope("/vehicles")
                    .service(vehicles::handlers::get_all_vehicles)
                    .service(vehicles::handlers::get_vehicle_detail)
                    .service(vehicles::handlers::get_featured_vehicles)
            )
            .service(
                web::scope("/admin/vehicles")
                    .wrap(auth::middleware::JwtAuth)
                    .wrap(auth::middleware::AdminRoleCheck)
                    .service(vehicles::handlers::create_vehicle)
                    .service(vehicles::handlers::update_vehicle)
                    .service(vehicles::handlers::delete_vehicle)
            )
            // Auctions
            .service(
                web::scope("/auctions")
                    .service(auctions::handlers::get_all_auctions)
                    .service(auctions::handlers::get_auction_detail)
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
            // Inquiries
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
            // Users
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

            .default_service(web::to(|| async { HttpResponse::NotFound().body("404") }))
    })
    .bind(bind_addr)?
    .run()
    .await
}
