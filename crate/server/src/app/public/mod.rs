pub mod login;
pub mod route;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct AuthPayload {
    pub pubkey: String,
    pub message: String,
    pub signature: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct Keys {
    authenticity_token: String,
}

pub(crate) async fn handler(token: csrf::CsrfToken) -> impl axum::response::IntoResponse {
    let keys = serde_json::json!(Keys {
        authenticity_token: token.authenticity_token(),
    });

    (token, format!("{keys}"))
}
