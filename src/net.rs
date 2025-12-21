use axum::Router;
use tower_http::services::ServeDir;

use crate::SharedCollection;

pub fn app() -> Router<SharedCollection> {
    let app = Router::new()
        .fallback_service(ServeDir::new("web"));
    
    app
}