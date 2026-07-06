use std::{path::Path, time::UNIX_EPOCH};

use axum_cookie::CookieManager;
use openidconnect::AccessToken;
use serde::{Deserialize, Serialize};

pub type SessionId = String;
#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub email: String,
    pub expiration: i64,
    pub access_token: AccessToken
}

impl Session {
    pub fn is_expired(&self) -> bool {
        let now = UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;

        now - self.expiration > 0
    }

    pub async fn retrieve(id: &SessionId) -> Option<Self> {
        let bucket = std::env::var("SESSION_BUCKET")
            .expect("SESSION_BUCKET should be supplied to use sessions");

        let path = Path::new(&bucket).join(id);
        if !path.exists() {
            return None
        }

        if let Ok(session) = tokio::fs::read(path).await {
            if let Ok(session) = toml::from_slice::<Session>(&session) {
                return Some(session)
            }
        }

        None
    }

    pub fn generate_id() -> SessionId {
        random_id(32)
    }

    pub async fn save(&self, id: &SessionId) -> bool {
        let bucket = std::env::var("SESSION_BUCKET")
            .expect("SESSION_BUCKET should be supplied to use sessions");

        let path = Path::new(&bucket).join(id);
        if path.exists() {
            return false
        }

        if let Ok(slice) = toml::to_string(&self) {
            if let Ok(()) = tokio::fs::write(path, slice).await {
                return true
            }
        }

        false
    }
    pub async fn from_cookie(cookies: &CookieManager) -> Option<(String, Session)> {
        let id = cookies
            .get("id")
            .and_then(|cookie| Some(cookie.value().to_string()));

        if let Some(id) = id {
            if let Some(session) = Session::retrieve(&id).await && !session.is_expired() {
                return Some((id, session))
            }
        }        
        None
    }

    pub async fn delete(id: &SessionId) -> Option<()> {
        let bucket = std::env::var("SESSION_BUCKET")
            .expect("SESSION_BUCKET should be supplied to use sessions");

        let path = Path::new(&bucket).join(id);
        if !path.exists() {
            return None
        }

        if let Ok(()) = tokio::fs::remove_file(path).await {
            return Some(())
        }

        None
    }
}

pub async fn filter_outdated_sessions() -> Result<(), tokio::io::Error> {
    let bucket = std::env::var("SESSION_BUCKET")
        .expect("SESSION_BUCKET should be supplied to use sessions");

    // iterate over every session and delete if it has expired
    for entry in std::fs::read_dir(&bucket)? {
        let entry = match entry {
            Ok(entry) => entry,
            // if can't read go to next
            Err(_) => continue
        };

        let path = entry.path();
        let id = path
            .strip_prefix(&bucket).unwrap()
            .to_str().unwrap()
            .to_string();

        if let Some(session) = Session::retrieve(&id).await && session.is_expired() {
            tokio::fs::remove_file(path).await?;
        }
    }

    Ok(())
}

fn random_id(n: u32) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
        abcdefghijklmnopqrstuvwxyz\
        0123456789";
        
    (0..n)
        .map(|_| CHARSET[rand::random_range(0..CHARSET.len())] as char)
        .collect()
}