use hyper::{body::HttpBody, client::HttpConnector, Client, Request};
use hyper_rustls::HttpsConnector;
use reqwest::header;
use rustls::{Certificate, ClientConfig, RootCertStore};
use rustls_pemfile::{read_one, Item};
use shared::config::path;
use std::io::BufReader;
use tokio::io::AsyncWriteExt;

use shared::model::Ressource;

async fn init_file(model_name: &str, ressource_name: &str) -> tokio::fs::File {
    let filedir = path::data().join(model_name);
    if !filedir.exists() {
        tokio::fs::create_dir(&filedir).await.ok().unwrap();
    }

    let filepath = path::data().join(model_name).join(ressource_name);
    if filepath.exists() {
        tokio::fs::remove_file(&filepath).await.ok().unwrap();
    }

    tokio::fs::File::create(&filepath).await.ok().unwrap()
}

pub async fn client() -> hyper::Client<HttpsConnector<HttpConnector>> {
    let mut root_store = RootCertStore::empty();

    let path = path::workspace().join("ssl").join("cli").join("gen.crt");
    let file = std::fs::File::open(path).ok().unwrap();
    let mut reader = BufReader::new(file);
    let cert = match read_one(&mut reader).transpose().unwrap().unwrap() {
        Item::X509Certificate(cert) => cert,
        _ => panic!("not cert"),
    };
    let rustls_cert = rustls::Certificate(cert);

    for cert in rustls_native_certs::load_native_certs().ok().unwrap() {
        root_store.add(&Certificate(cert.0)).ok().unwrap();
    }
    root_store.add(&rustls_cert).ok().unwrap();

    let client_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(client_config)
        .https_only()
        .enable_http2()
        .build();

    Client::builder()
        .http2_only(true)
        .build::<_, hyper::Body>(connector)
}

pub async fn download(
    multibar: indicatif::MultiProgress,
    model_name: &str,
    ressource: Ressource,
) -> Result<(), &'static str> {
    let client = client().await;
    let req = Request::get(&ressource.url)
        .body(hyper::Body::empty())
        .ok()
        .unwrap();
    let res = client.request(req).await.ok().unwrap();

    let download_size = res
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|ct_len| ct_len.to_str().ok())
        .and_then(|ct_len| ct_len.parse::<u64>().ok())
        .unwrap();

    let mut file = init_file(model_name, &ressource.name).await;

    let progress_bar = multibar.add(indicatif::ProgressBar::new(download_size));

    progress_bar.set_style(
        indicatif::ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let set_fmt = |downloaded: u64| {
        format!(
            "{:?}, downloaded: {downloaded}, ratio: {}",
            ressource.name,
            downloaded / download_size * 100u64
        )
    };

    let mut counter = 0u64;
    let mut downloaded = 0u64;
    let mut body = res.into_body();
    while let Some(chunk) = body
        .data()
        .await
        .transpose()
        .map_err(|_| "chunk read failed")?
    {
        file.write_all(&chunk)
            .await
            .map_err(|_| "chunk write failed")?;

        downloaded += chunk.len() as u64;
        counter += 1;
        if counter % 10 == 9 {
            progress_bar.set_message(set_fmt(downloaded));
        }
        progress_bar.inc(chunk.len() as u64);
    }
    file.flush().await.ok().unwrap();

    progress_bar.println(format!("Loading of URL: {:?} is done!", ressource.url));

    Ok(())
}
