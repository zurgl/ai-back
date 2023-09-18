use axum::{
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde_json::json;

pub async fn handler() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("Logged out");
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(time::Duration::hours(-1))
        .secure(true)
        .same_site(SameSite::None)
        .http_only(false)
        .finish();

    let mut response = Response::new(json!({"status": "success"}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}
