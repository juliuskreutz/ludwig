use std::{fs, time::Duration};

use acme2::{AccountBuilder, Csr, DirectoryBuilder, Error, OrderBuilder};
use actix_files::Files;
use actix_web::{rt::spawn, App, HttpServer};
use rustls::{Certificate, PrivateKey, ServerConfig};

pub async fn config() -> ServerConfig {
    fs::create_dir_all(".well-known/acme-challenge/").unwrap();

    let acme = spawn(
        HttpServer::new(|| {
            App::new().service(Files::new(
                "/.well-known/acme-challenge/",
                ".well-known/acme-challenge/",
            ))
        })
        .bind(("0.0.0.0", 80))
        .unwrap()
        .run(),
    );

    let (key, certs) = certificates().await.unwrap();

    acme.abort();
    fs::remove_dir_all(".well-known/").unwrap();

    ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap()
}

async fn certificates() -> Result<(PrivateKey, Vec<Certificate>), Error> {
    let dir = DirectoryBuilder::new("https://acme-v02.api.letsencrypt.org/directory".to_string())
        .build()
        .await?;

    let mut builder = AccountBuilder::new(dir.clone());
    builder.contact(vec!["mailto:julius@kreutz.dev".to_string()]);
    builder.terms_of_service_agreed(true);
    let account = builder.build().await?;

    let mut builder = OrderBuilder::new(account);
    builder.add_dns_identifier("spund.de".to_string());
    let order = builder.build().await?;

    for auth in order.authorizations().await? {
        let challenge = auth.get_challenge("http-01").unwrap();

        let token = challenge.token.clone().unwrap();
        let contents = challenge.key_authorization()?.clone().unwrap();

        fs::write(format!(".well-known/acme-challenge/{token}"), contents).unwrap();

        let challenge = challenge.validate().await?;

        challenge.wait_done(Duration::from_secs(5), 3).await?;

        auth.wait_done(Duration::from_secs(5), 3).await?;
    }

    let order = order.wait_ready(Duration::from_secs(5), 3).await?;

    let pkey = acme2::gen_rsa_private_key(4096)?;

    let key = PrivateKey(pkey.private_key_to_der().unwrap());

    let order = order.finalize(Csr::Automatic(pkey)).await?;

    let order = order.wait_done(Duration::from_secs(5), 3).await?;

    let certs = order
        .certificate()
        .await?
        .unwrap()
        .iter()
        .map(|vec| Certificate(vec.to_der().unwrap()))
        .collect();

    Ok((key, certs))
}
