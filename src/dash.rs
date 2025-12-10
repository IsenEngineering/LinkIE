use std::collections::HashMap;

use axum::{Router, extract::{Json, Request, State}, http::{HeaderMap, StatusCode, header}, middleware::{self, Next}, response::{IntoResponse, Response}, routing::{get, get_service}};
use serde::Deserialize;
use tower_http::services::ServeFile;
use crate::{auth::{Auth, AuthDriver}, collection::{Collection, Link}};

#[derive(Deserialize, Debug)]
struct FormPayload {
    user: String,
    pass: String
}

fn extract_token(headers: &HeaderMap) -> Result<String, ()> {
    if let Some(cookie_header) = headers.get("cookie") {
        for pair in cookie_header.to_str().unwrap().split("; ") {
            let mut parts = pair.splitn(2, '=');
            let name = parts.next();
            let value = parts.next();

            if name == Some("token") {
                let jwt = value.unwrap().to_string();
                return Ok(jwt)
            }
        }
    }
    Err(())
}

async fn login(Json(body): Json<FormPayload>) -> Response {
    match Auth::login(body.user, body.pass).await {
        Ok(jwt) => {
            let set_cookie = format!("token={}; Max-Age={}", jwt, 60 * 60 * 24 * 3);

            (StatusCode::OK, [(header::SET_COOKIE, set_cookie)], "ok".to_string()).into_response()
        },
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response()
    }
}

async fn authentificated(req: Request, next: Next) -> Response {
    match extract_token(req.headers()) {
        Ok(jwt) => match Auth::authentificate(jwt).await {
            Ok(()) => next.run(req).await,
            Err(e) => (StatusCode::FORBIDDEN, e).into_response()
        }
        Err(()) => (StatusCode::FORBIDDEN, "Forbidden").into_response()
    }
} 

async fn collection(State(collection): State<Collection>) -> Json<HashMap<String, Link>> {
    let map = collection.list();

    Json(map)
}

pub fn routes() -> Router<Collection> {
    Router::new()
        .route_service("/", 
            get_service(ServeFile::new("./web/login.html"))
            .post(login))
        .route_service("/dash", 
            get_service(ServeFile::new("./web/dash.html"))
                .layer(middleware::from_fn(authentificated)))
        .route("/dash/collection", 
            get(collection)
                .layer(middleware::from_fn(authentificated))
        )
}