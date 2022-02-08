use std::io;

use actix_web::{get, App, HttpServer, Responder};

mod ssl;

#[get("/")]
async fn index() -> impl Responder {
    "Hello World"
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let ssl = ssl::config().await;

    HttpServer::new(move || App::new().service(index))
        .bind(("0.0.0.0", 80))?
        .bind_rustls(("0.0.0.0", 443), ssl)?
        .run()
        .await
}
