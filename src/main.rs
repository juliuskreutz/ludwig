use std::io;

use actix_files::Files;
use actix_session::CookieSession;
use actix_web::{web::Data, App, HttpServer};
use handlebars::Handlebars;
use https::Https;
use rand::Rng;

mod auth;
mod files;
mod https;
mod ssl;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let ssl = ssl::config().await;

    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", "templates")
        .unwrap();

    let key = rand::thread_rng().gen::<[u8; 32]>();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(handlebars.clone()))
            .wrap(Https::new())
            .wrap(CookieSession::private(&key).name("auth"))
            .service(Files::new("/static/", "static"))
            .configure(auth::config)
            .configure(files::config)
    })
    .bind(("0.0.0.0", 80))?
    .bind_rustls(("0.0.0.0", 443), ssl)?
    .run()
    .await
}
