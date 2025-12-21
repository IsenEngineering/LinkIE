use axum::{extract::Json, http::{HeaderValue, StatusCode, header}, response::{IntoResponse, Redirect, Response}};
use serde::Deserialize;
use tower_http::services::ServeFile;
use crate::{auth::{Auth, AuthDriver}};

#[derive(Deserialize, Debug)]
pub struct FormPayload {
    user: String,
    pass: String
}

pub async fn login(Json(body): Json<FormPayload>) -> Response {
    match Auth::login(body.user, body.pass).await {
        Ok(jwt) => {
            let set_cookie = format!("token={}; Max-Age={}", jwt, 60 * 60 * 24 * 3);

            (StatusCode::OK, [(header::SET_COOKIE, set_cookie)], "ok".to_string()).into_response()
        },
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response()
    }
}

pub fn login_page() -> ServeFile {
    ServeFile::new("./web/login.html")
}
pub fn dashboard_page() -> ServeFile {
    ServeFile::new("./web/dash.html")
}

pub async fn logout() -> Response {
    let mut redirection = Redirect::to("/").into_response();
    let headers = redirection.headers_mut();
    headers.insert("Set-Cookie", HeaderValue::from_static("token=; Max-Age=0"));

    redirection
}