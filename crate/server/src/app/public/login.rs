use std::sync::Arc;

use axum::{
    http::{header, Response, StatusCode},
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;

use crate::db::model::{TokenClaims, User};

use super::{route::State, AuthPayload};

pub(crate) async fn handler(
    token: csrf::CsrfToken,
    Extension(state): Extension<Arc<State>>,
    body: axum::Json<AuthPayload>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if token.verify(&body.token).is_ok() {
        let msg = format!("{}{}", body.message, body.token);
        let signature = bs58::decode(&body.signature).into_vec().ok().unwrap();
        let pubkey = bs58::decode(&body.pubkey).into_vec().ok().unwrap();

        if !nacl::sign::verify(&signature, msg.as_bytes(), &pubkey)
            .ok()
            .unwrap_or(false)
        {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Cannot verify msg token."),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        } else {
            let user = sqlx::query_as!(
                User,
                "SELECT * FROM users WHERE pubkey = $1;",
                body.pubkey.to_string()
            )
            .fetch_optional(state.db.as_ref())
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "error",
                    "message": format!("Database error: {}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

            let id = match user {
                Some(user) => user.id,
                None => {
                    let user = sqlx::query_as!(
                        User,
                        "INSERT INTO users (pubkey) VALUES ($1) RETURNING *",
                        body.pubkey.to_string(),
                    )
                    .fetch_one(state.db.as_ref())
                    .await
                    .map_err(|e| {
                        let error_response = serde_json::json!({
                            "status": "fail",
                            "message": format!("Database error: {}", e),
                        });
                        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                    })?;
                    user.id
                }
            };

            let now = chrono::Utc::now();
            let iat = now.timestamp() as usize;
            let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;
            let claims: TokenClaims = TokenClaims {
                sub: id.to_string(),
                exp,
                iat,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.env.jwt_secret.as_ref()),
            )
            .unwrap();

            let cookie = Cookie::build("token", token.to_owned())
                .path("/")
                .max_age(time::Duration::hours(1))
                .secure(true)
                .same_site(SameSite::None)
                .http_only(false)
                .finish();

            let mut response =
                Response::new(json!({"status": "success", "token": token}).to_string());
            response
                .headers_mut()
                .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

            Ok(response)
        }
    } else {
        let error_response = serde_json::json!({
          "status": "fail",
          "message": "Invalid csrf",
        });
        Err((StatusCode::BAD_REQUEST, Json(error_response)))
    }
}
