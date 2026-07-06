use std::time::{Duration, UNIX_EPOCH};

use axum_cookie::{CookieManager, cookie::Cookie};
use reqwest::StatusCode;
use serde::Deserialize;
use axum::{Router, extract::{Query, State}, response::{IntoResponse, Redirect, Response}};
use openidconnect::{
    AuthorizationCode, CsrfToken, Nonce, OAuth2TokenResponse, Scope, core::{CoreResponseType, CoreRevocableToken}, reqwest::{Client, redirect::Policy}
};

use super::config::{OidcConfig, OidcToken, OidcClient};
use super::state::Auth;
use super::Session;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", axum::routing::get(login))
        .route("/auth/validate", axum::routing::get(validate))
        .route("/auth/logout", axum::routing::get(logout))
}

// /auth/login
async fn login(State(mut state): State<Auth>, cookies: CookieManager) -> Redirect {
    if let Some(_session) = Session::from_cookie(&cookies).await {
        // prevent from redirecting to google if already authentificated
        return Redirect::to("/dash");
    }

    let client = OidcConfig::new().client().await;


    // authorization url generation
    let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            openidconnect::AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
        .url();

    // let's store in memory the state for further exchange with the client
    state.insert(csrf_state, nonce).await;

    // redirect clients to the identity provider to authentificate
    Redirect::to(authorize_url.as_str())
}

#[derive(Deserialize)]
pub struct ValidationParams {
    pub code: AuthorizationCode,
    pub state: CsrfToken
}

// /auth/validate
async fn validate(State(mut state): State<Auth>, cookies: CookieManager, params: Query<ValidationParams>) -> Result<Response, (StatusCode, String)> {
    // let's compare if the given state corresponds to any state in memory and remove it
    let pair = match state.check_remove(&params.state).await {
        Some(pair) => pair,
        None => return Err((
            StatusCode::BAD_REQUEST,
            "There is no session for this token".to_string()
        ))
    };
    
    // let's exchange code against a token to the identity provider
    let client = OidcConfig::new().client().await;
    let token = exchange_code_against_token(&client, params.code.clone()).await?;
    
    let id_token = token.extra_fields().id_token();
    
    let id_token_verifier = client.id_token_verifier();
    let id_token_claims = match id_token.unwrap().claims(&id_token_verifier, &pair.1) {
        Ok(claims) => claims,
        Err(e) => return Err((StatusCode::UNAUTHORIZED, e.to_string()))
    };

    
    let expiration = id_token_claims.expiration().timestamp();
    let email = id_token_claims.email()
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "email not in claims".to_string()))?
        .to_string();

    println!("token granted for `{}`", &email);
    let session_id = Session::generate_id();
    let session = Session {
        expiration,
        email,
        access_token: token.access_token().clone()
    };

    if !session.save(&session_id).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Server failed to save session".to_string()));
    }

    let redirection = Redirect::to("/dash").into_response();
    let age: Duration = Duration::from_secs(expiration as u64) - UNIX_EPOCH.elapsed().unwrap();
    
    let cookie = Cookie::new("id", session_id)
        .with_domain("localhost")
        .with_max_age(age)
        .with_path("/")
        .with_same_site(axum_cookie::cookie::cookie::SameSite::Strict);

    cookies.set(cookie);
    
    Ok(redirection)
}

async fn exchange_code_against_token(client: &OidcClient, code: AuthorizationCode) -> Result<OidcToken, (StatusCode, String)> {
    // let's exchange code against a token to the identity provider
    let token_request = match client.exchange_code(code) {
        Ok(req) => req,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    };

    let http_client = Client::builder()
        // prevents SSRF
        .redirect(Policy::none())
        .build().unwrap();

    match token_request.request_async(&http_client).await {
        Ok(token) => Ok(token),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string()))
    }
}

async fn logout(cookies: CookieManager) -> Result<Redirect, (StatusCode, String)> {
    let session = Session::from_cookie(&cookies).await;
    if let Some((id, session)) = session {
        let client = OidcConfig::new().client().await;
            
        let http_client = Client::builder()
            // prevents SSRF
            .redirect(Policy::none())
            .build().unwrap();

        let token_to_revoke: CoreRevocableToken = session.access_token.into();
            
        client
            .revoke_token(token_to_revoke)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .request_async(&http_client).await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        cookies.remove("id");
        Session::delete(&id).await;
        println!("token revoked for `{}`", session.email)
    }

    Ok(Redirect::to("/"))
}