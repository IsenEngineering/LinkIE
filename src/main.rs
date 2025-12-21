use std::sync::Arc;
use axum::middleware;
use tokio::sync::RwLock;
use crate::collection::Collection;

pub type SharedCollection = Arc<RwLock<Collection>>;

mod collection;
mod net;
mod auth;
mod redirect;
mod dash;

#[tokio::main]
async fn main() {
    let collection = Collection::new(None);
    let shared_collection = Arc::new(RwLock::new(collection));
    let dash = dash::routes();
    let app = net::app()
        .merge(dash)
        .layer(middleware::from_fn_with_state(shared_collection.clone(), redirect::dash_middleware))
        .with_state(shared_collection);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("listening on localhost:80  📡...");
    axum::serve(listener, app).await.unwrap();
}