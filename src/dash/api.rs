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
    if payload.subdomain.is_none() && payload.path.is_none() {
        return Err("Impossible de créer une redirection sans sous-domaine ou chemin")
    }

    collection.new_pair((payload.subdomain, payload.path), payload.destination).await;
    
    Ok((StatusCode::CREATED, "Redirection créée"))
}

async fn update_link(
    State(mut collection): State<Collection>,
    Json(payload): Json<UpdateRequest>,
) -> Result<&'static str, (StatusCode, &'static str)> {
    let key = (payload.subdomain, payload.path);

    if let Some(dest) = payload.new_destination && 
        let None = collection.update_destination(&key, dest).await {
        return Err((StatusCode::NOT_FOUND, "Cette redirection n'existe pas"))
    }

    if payload.new_path.is_some() || payload.new_subdomain.is_some() {
        let new_key = (payload.new_subdomain, payload.new_path);
        if let None = collection.update_key(&key, new_key).await {
            return Err((StatusCode::NOT_FOUND, "Cette redirection n'existe pas"))
        }
    }

    Ok("Ok")
}

async fn delete_link(
    State(mut collection): State<Collection>,
    Json(payload): Json<DeleteRequest>,
) -> Result<&'static str, (StatusCode, String)> {
    let resp = match collection.remove((payload.subdomain, payload.path)).await {
        Some(()) => Ok("Supprimé"),
        None => Err((StatusCode::NOT_FOUND, "Cette redirection n'existe pas".to_string()))
    };

    if let Err(e) = collection.save().await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }

    resp
}

pub fn endpoints(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/links", get(list_links))
        .route("/link", post(create_link))
        .route("/link", patch(update_link))
        .route("/link", delete(delete_link))
        .layer(middleware::from_fn_with_state(state, middlewares::authentificated))
}