mod middlewares;
mod routes;
mod api;

use axum::{Router, middleware, routing::{get, get_service}};
use crate::SharedCollection;

pub fn routes() -> Router<SharedCollection> {
    Router::new()
        .route_service("/", 
            get_service(routes::login_page())
            .post(routes::login)
            .layer(middleware::from_fn(middlewares::redirect_if_authentificated)))
        .route_service("/dash", 
            get_service(routes::dashboard_page())
                .layer(middleware::from_fn(middlewares::authentificated)))
        .route("/logout", get(routes::logout))
        .nest("/api", api::endpoints())
}