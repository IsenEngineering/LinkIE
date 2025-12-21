use axum::{
    extract::{Request, State}, 
    http::{HeaderMap, request::Parts},
    middleware::Next, 
    response::{IntoResponse, Redirect, Response}
};
use crate::{SharedCollection, net::not_found};

fn map_path(path: &str) -> Option<String> {
    let path = path.strip_prefix("/").unwrap().to_string();
    match path.len() {
        0 => None,
        _ => Some(path)
    }
}
  
fn map_host(headers: &HeaderMap) -> Option<String> {
    let header = headers.get("host");
    if header.is_none() { return None }

    let header = header.unwrap().to_str();
    if header.is_err() { return None }

    let host = header.unwrap();

    if host == "isenengineering.fr" { return None }
    
    host.strip_suffix(".isenengineering.fr")
        .and_then(|subdomain| Some(subdomain.to_string()))
}
async fn redirect(collection: SharedCollection, parts: Parts) -> Response {
    let path = map_path(parts.uri.path());
    let host = map_host(&parts.headers);
    let key = (host, path);

    match collection.read().await.find(&key) {
        Some(redirection) => {
            Redirect::to(&redirection).into_response()
        },
        None => not_found().into_response()
    }
}

pub async fn dash_middleware(
    State(collection): State<SharedCollection>,
    request: Request,
    next: Next,
) -> Response {
    let dashboard_host = vec!["link-ie.isenengineering.fr", "localhost"];
    
    let host = request
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok());
    
    match host {
        Some(h) if dashboard_host.iter().any(|&allowed| h.starts_with(allowed)) => {
            // dashboard response
            next.run(request).await
        }
        _ => {
            let (parts, _) = request.into_parts();

            // redirection using collection
            redirect(collection, parts).await
        }
    }
}