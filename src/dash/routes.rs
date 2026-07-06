use tower_http::services::ServeFile;
use axum::{Router, middleware, routing::get_service};
use crate::AppState;
use super::{api, middlewares};

pub fn login_page() -> ServeFile {
    ServeFile::new("./web/login.html")
}
pub fn dashboard_page() -> ServeFile {
    ServeFile::new("./web/dash.html")
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route_service("/", 
            get_service(login_page())
            .layer(middleware::from_fn_with_state(state.clone(), middlewares::redirect_if_authentificated))
            )
        .route_service("/dash", 
            get_service(dashboard_page())
                .layer(middleware::from_fn_with_state(state.clone(), middlewares::authentificated))
                )
        .nest("/api", api::endpoints(state))
}