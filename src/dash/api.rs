use std::collections::HashMap;

use axum::{Json, Router, extract::State, middleware, routing::{delete, get, patch, post}};
use reqwest::StatusCode;
use serde::Deserialize;
use crate::{AppState, dash::{collection::{Collection, Link}, middlewares}};

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub subdomain: Option<String>,
    pub path: Option<String>,
    pub destination: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub subdomain: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    pub subdomain: Option<String>,
    pub path: Option<String>,
    pub new_subdomain: Option<String>,
    pub new_path: Option<String>,
    pub new_destination: Option<String>,
}

async fn list_links(State(collection): State<Collection>) -> Json<HashMap<String, Link>> {
    Json(collection.list().await)
}

async fn create_link(
    State(mut collection): State<Collection>,
    Json(payload): Json<CreateRequest>,
) -> Result<(StatusCode, &'static str), &'static str> {
    match (payload.subdomain, payload.path) {
        (Some(subdomain), Some(path)) => {
            collection.new_subdomain_with_path(subdomain, path, payload.destination).await;
        },
        (None, Some(path)) => {
            collection.new_path(path, payload.destination).await;
        },
        (Some(subdomain), None) => {
            collection.new_subdomain(subdomain, payload.destination).await;
        },
        (None, None) => {
            return Err("Impossible de créer une redirection sans sous-domaine ou chemin")
        }
    }
    
    Ok((StatusCode::CREATED,"Redirection créée"))
}

async fn update_link(
    State(mut collection): State<Collection>,
    Json(payload): Json<UpdateRequest>,
) -> Result<&'static str, (StatusCode, &'static str)> {
    let key = (payload.subdomain, payload.path);
    let id = collection.find_id(&key).await;

    if id.is_none() {
        return Err((StatusCode::NOT_FOUND, "Cette redirection n'existe pas"))
    }
    
    if let Some(dest) = payload.new_destination &&
        let Err(()) = collection.update_destination(&key, dest).await {
        // en théorie cette condition ne sera jamais remplie
        return Err((StatusCode::NOT_FOUND, "Cette redirection n'existe pas"))
    }

    if payload.new_path.is_some() || payload.new_subdomain.is_some() {
        collection.update_key(&key, (payload.new_subdomain, payload.new_path)).await.unwrap();
    }

    Ok("Ok")
}

async fn delete_link(
    State(mut collection): State<Collection>,
    Json(payload): Json<DeleteRequest>,
) -> Result<&'static str, (StatusCode, &'static str)> {
    let key = (payload.subdomain, payload.path);

    match collection.remove(key).await {
        Ok(()) => Ok("Supprimé"),
        Err(()) => Err((StatusCode::NOT_FOUND, "Cette redirection n'existe pas"))
    }
}

pub fn endpoints(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/links", get(list_links))
        .route("/link", post(create_link))
        .route("/link", patch(update_link))
        .route("/link", delete(delete_link))
        .layer(middleware::from_fn_with_state(state, middlewares::authentificated))
}