use std::io;

use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie, web::Data, App, HttpServer};
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

    let mut key = [0; 64];
    rand::thread_rng().fill(&mut key);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(handlebars.clone()))
            .wrap(Https::new())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                cookie::Key::from(&key),
            ))
            .service(Files::new("/static/", "static"))
            .configure(auth::config)
            .configure(files::config)
    })
    .bind(("0.0.0.0", 80))?
    .bind_rustls(("0.0.0.0", 443), ssl)?
    .run()
    .await
}
