use actix_session::Session;
use actix_web::{
    get, post,
    web::{Form, ServiceConfig},
    HttpResponse, Responder,
};
use argon2::Config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    name: String,
    password: String,
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(login).service(login_post).service(logout);
}

#[get("/login")]
async fn login(session: Session) -> impl Responder {
    if let Some(auth) = session.get::<String>("auth").unwrap() {
        if serde_json::from_str::<User>(&auth).is_ok() {
            return HttpResponse::Found()
                .append_header(("LOCATION", "/"))
                .finish();
        }
    }

    session.clear();

    HttpResponse::Ok().body(include_str!("../templates/login.html"))
}

#[post("/login")]
async fn login_post(session: Session, user: Form<User>) -> impl Responder {
    let hash = encode(&user.password);

    if user.name == "ludwig" && hash == "$argon2i$v=19$m=4096,t=3,p=1$cmFuZG9tX3NhbHQ$hy47/Y18TGiOEB0d2pZNAAmIwJ6czkQSQsSlNcvc468" {
        session
            .insert("auth", serde_json::to_string(&user).unwrap())
            .unwrap();
    }

    HttpResponse::Found()
        .append_header(("LOCATION", "/"))
        .finish()
}

#[get("/logout")]
async fn logout(session: Session) -> impl Responder {
    session.clear();

    HttpResponse::Found()
        .append_header(("LOCATION", "/"))
        .finish()
}

pub fn is_ludwig(user: &User) -> bool {
    user.name == "ludwig" && encode(&user.password) == "$argon2i$v=19$m=4096,t=3,p=1$cmFuZG9tX3NhbHQ$hy47/Y18TGiOEB0d2pZNAAmIwJ6czkQSQsSlNcvc468"
}

fn encode(password: &str) -> String {
    argon2::hash_encoded(password.as_bytes(), b"random_salt", &Config::default()).unwrap()
}
