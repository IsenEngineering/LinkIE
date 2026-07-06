use std::{sync::Arc, time::Duration};

use axum::extract::FromRef;
use openidconnect::{CsrfToken, Nonce};
use tokio::{sync::RwLock, time::Instant};

use crate::AppState;

// Pour chaque authentification le serveur doit garder en mémoire des valeurs pour assurer l'identité entre tous les échanges
// On supprime les valeurs après 5 minutes toutes les 300 secondes
#[derive(Clone)]
pub struct Auth(pub Arc<RwLock<Vec<(CsrfToken, Nonce, Instant)>>>);

impl Auth {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(vec![])))
    }

    // filter for every outdated pair of csrf token and nonce
    pub async fn filter_outdated(&mut self) {
        let mut guard = self.0.write().await;
        guard.retain_mut(|p| 
            p.2.duration_since(Instant::now()) < Duration::from_secs(300));
    }

    // inserts a pair of csrf token and nonce within the struct
    pub async fn insert(&mut self, csrf: CsrfToken, nonce: Nonce) {
        let mut state = self.0.write().await;
    
        state.push((csrf, nonce, Instant::now()));
    }

    // checks whether a csrf token is within the struct and removes it
    pub async fn check_remove(&mut self, csrf: &CsrfToken) -> Option<(CsrfToken, Nonce)> {
        let guard = self.0.read().await;

        let pair: Option<(CsrfToken, Nonce, Instant)> = guard
            .iter()
            .find(|pair| pair.0.secret() == csrf.secret())
            .cloned();

        // dropping read guard
        drop(guard);
        // asking write guard

        match pair {
            Some(pair) => {
                // remove the pair from the memory
                let mut guard = self.0.write().await;
                // remove the pair from the memory
                guard.retain_mut(|item| item.0.secret() == pair.0.secret());

                Some((pair.0, pair.1))
            },
            None => None
        }
    }
}

impl FromRef<AppState> for Auth {
    fn from_ref(input: &AppState) -> Self {
        input.auth.clone()
    }
}