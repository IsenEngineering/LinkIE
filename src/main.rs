use std::time::Duration;

use axum_cookie::CookieLayer;
use tokio::time::interval;

use crate::{auth::Auth, dash::Collection};

mod net;
mod redirect;
mod dash;
mod auth;

#[derive(Clone)]
pub struct AppState {
    pub collection: Collection,
    pub auth: Auth
}

// use auth::filter_outdated_sessions

const SESSION_CLEANUP: Duration = Duration::from_secs(7200);
const AUTH_CLEANUP: Duration = Duration::from_secs(300);

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let state = AppState {
        collection: Collection::new(None),
        auth: Auth::new()
    };

    let mut moved_auth = state.auth.clone();
    tokio::spawn(async move { 
        let mut session_i = interval(SESSION_CLEANUP);
        session_i.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        
        let mut auth_i = interval(AUTH_CLEANUP);
        auth_i.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                // Every 2h, we remove expired sessions from disk
                _ = session_i.tick() => {
                    auth::filter_outdated_sessions().await.unwrap();
                },
                // Every 5min, we remove expired authentification transactions from memory
                _ = auth_i.tick() => {
                    moved_auth.filter_outdated().await;
                }
            }
        }
    });

    let dash_routes = dash::routes(state.clone());
    let auth_routes = auth::routes();
    let app = net::app()
        .merge(auth_routes)
        .merge(dash_routes)
        .layer(CookieLayer::default())
        .layer(axum::middleware::from_fn_with_state(state.clone(), redirect::dash_middleware))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("listening on localhost:80  📡...");
    axum::serve(listener, app).await.unwrap();
}