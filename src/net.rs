use axum::{Router, response::IntoResponse, http::StatusCode};
use tower_http::services::ServeDir;
use crate::SharedCollection;

const NOT_FOUND: &str = include_str!("../web/404.html");
pub fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, [("Content-Type", "text/html")], NOT_FOUND)
}

pub fn app() -> Router<SharedCollection> {
    let app = Router::new()
        .fallback_service(ServeDir::new("web"));
    
    app
}