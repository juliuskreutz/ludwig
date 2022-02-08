use std::{
    fs::File,
    io::{self, BufReader},
};

use actix_web::{App, HttpServer};
use rustls::{Certificate, PrivateKey, ServerConfig};

#[actix_web::main]
async fn main() -> io::Result<()> {
    let ssl = load_ssl();

    HttpServer::new(move || App::new().service(actix_files::Files::new("/static", "static")))
        .bind(("0.0.0.0", 80))?
        .bind_rustls(("0.0.0.0", 443), ssl)?
        .run()
        .await
}

fn load_ssl() -> ServerConfig {
    let config = ServerConfig::builder();
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());
    let cert_chain = rustls_pemfile::certs(cert_file)
        .unwrap()
        .iter()
        .map(|vec| Certificate(vec.clone()))
        .collect();
    let mut keys = rustls_pemfile::pkcs8_private_keys(key_file).unwrap();

    config
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, PrivateKey(keys.remove(0)))
        .unwrap()
}
