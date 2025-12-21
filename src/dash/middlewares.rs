use axum::{extract::Request, http::HeaderMap, middleware::Next, response::{IntoResponse, Redirect, Response}};
use crate::auth::{Auth, AuthDriver};

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
pub async fn authentificated(req: Request, next: Next) -> Response {
    match extract_token(req.headers()) {
        Ok(jwt) => match Auth::authentificate(jwt).await {
            Ok(()) => next.run(req).await,
            Err(_) => Redirect::to("/").into_response()
        }
        Err(()) => Redirect::to("/").into_response()
    }
} 

pub async fn redirect_if_authentificated(req: Request, next: Next) -> Response {
    match extract_token(req.headers()) {
        Ok(jwt) => match Auth::authentificate(jwt).await {
            Ok(()) => Redirect::to("/dash").into_response(),
            Err(_) => next.run(req).await
        }
        Err(()) => next.run(req).await
    }
}