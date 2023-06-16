use std::path::PathBuf;

use axum_server::tls_rustls::RustlsConfig;

pub fn workspace() -> PathBuf {
    let current_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    current_path.ancestors().nth(2).unwrap().to_path_buf()
}

pub async fn config_load(is_local: bool) -> RustlsConfig {
    match is_local {
        true => RustlsConfig::from_pem_file(
            workspace().join("ssl").join("server").join("CA.crt"),
            workspace().join("ssl").join("server").join("CA.key"),
        )
        .await
        .unwrap(),
        false => RustlsConfig::from_pem_file(
            workspace().join("ssl").join("server").join("ionos.crt"),
            workspace().join("ssl").join("server").join("ionos.key"),
        )
        .await
        .unwrap(),
    }
}
