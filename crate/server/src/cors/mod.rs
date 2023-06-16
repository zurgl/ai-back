use axum::http::header::{
    ACCEPT, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
    ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_HEADERS, AUTHORIZATION, CONTENT_TYPE,
    COOKIE,
};
use axum::http::Method;
use tower_http::cors::CorsLayer;

use shared::constants;

pub fn load() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            COOKIE,
            ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_ALLOW_CREDENTIALS,
            ACCESS_CONTROL_REQUEST_HEADERS,
            CONTENT_TYPE,
            AUTHORIZATION,
            ACCEPT,
        ])
        .allow_origin(constants::server::CORS_ALLOWED_ORIGINS.map(|origin| origin.parse().unwrap()))
        .allow_credentials(true)
}
