use actix_web::{get, App, HttpServer, Responder};

#[get("/")]
async fn health() -> impl Responder {
    "Manga Automobiles API up and running!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting backend on http://127.0.0.1:8080");
    HttpServer::new(|| App::new().service(health))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
