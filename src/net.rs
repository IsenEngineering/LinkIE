use axum::Router;
use tower_http::services::ServeDir;

use crate::collection::Collection;

pub fn app() -> Router<Collection> {
    let app = Router::new()
        .fallback_service(ServeDir::new("web"));
    
    app
}