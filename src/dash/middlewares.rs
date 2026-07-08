use axum::{extract::{FromRequestParts, Request, State}, middleware::Next, response::{IntoResponse, Redirect, Response}};
use axum_cookie::CookieManager;

use crate::{AppState, auth::Session};

pub async fn authentificated(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let cookies = CookieManager::from_request_parts(&mut parts, &state).await;

    if let Ok(cookies) = cookies {
        let session = Session::from_cookie(&cookies).await;
        if let Some((_id, _session)) = session {
            let req = Request::from_parts(parts, body);
                                    
            return next.run(req).await
        }
    }

    if parts.uri.path().starts_with("/link") {
        return axum::http::StatusCode::UNAUTHORIZED.into_response()
    } else {
        Redirect::to("/").into_response()
    }
} 

pub async fn redirect_if_authentificated(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let cookies = CookieManager::from_request_parts(&mut parts, &state).await;

    if let Ok(cookies) = cookies {
        let session = Session::from_cookie(&cookies).await;
        if let Some((_id, _session)) = session {
            return Redirect::to("/dash").into_response()
        }
    }
    
    let req = Request::from_parts(parts, body);
    return next.run(req).await
}