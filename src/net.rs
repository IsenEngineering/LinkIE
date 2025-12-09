use axum::Router;
use tower_http::services::ServeDir;

pub fn app() -> Router {
    let app = Router::new()
        .fallback_service(ServeDir::new("web"));
    
    app
}

pub async fn serve(app: Router) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("listening on localhost:80  📡...");
    axum::serve(listener, app).await.unwrap();
}