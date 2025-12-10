use crate::collection::Collection;

mod collection;
mod net;
mod auth;
mod redirect;
mod dash;

#[tokio::main]
async fn main() {
    let collection = Collection::new(None);
    let dash = dash::routes();
    let app = net::app()
        .merge(dash)
        .with_state(collection);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("listening on localhost:80  📡...");
    axum::serve(listener, app).await.unwrap();
}