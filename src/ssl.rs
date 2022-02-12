use std::{fs, sync::mpsc, time::Duration};

use acme2::{AccountBuilder, Csr, DirectoryBuilder, Error, OrderBuilder};
use actix_files::Files;
use actix_web::{rt, App, HttpServer};
use rustls::{Certificate, PrivateKey, ServerConfig};

pub async fn config() -> ServerConfig {
    fs::create_dir_all(".well-known/acme-challenge/").unwrap();

    let (s, r) = mpsc::channel();

    rt::spawn({
        let server = HttpServer::new(|| {
            App::new().service(Files::new(
                "/.well-known/acme-challenge/",
                ".well-known/acme-challenge/",
            ))
        })
        .bind(("0.0.0.0", 80))
        .unwrap()
        .run();

        s.send(server.handle()).unwrap();

        server
    });

    let (key, certs) = if let Ok((key, certs)) = certificates().await {
        let _ = fs::remove_dir_all("certs");
        fs::create_dir("certs").unwrap();

        fs::write("certs/key", &key.0).unwrap();

        for (i, cert) in certs.iter().enumerate() {
            fs::write(format!("certs/cert{}", i), &cert.0).unwrap();
        }

        (key, certs)
    } else {
        let key = PrivateKey(fs::read("certs/key").unwrap());

        let mut certs = Vec::new();

        let mut i = 0;

        while let Ok(cert) = fs::read(format!("certs/cert{}", i)) {
            certs.push(Certificate(cert));
            i += 1;
        }

        (key, certs)
    };

    let handle = r.recv().unwrap();
    handle.stop(false).await;

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
    builder.add_dns_identifier("ludwig-frommelt.de".to_string());
    builder.add_dns_identifier("www.ludwig-frommelt.de".to_string());
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
