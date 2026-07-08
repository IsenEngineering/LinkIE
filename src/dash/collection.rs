use std::{collections::HashMap, env, fs, sync::Arc};

use axum::extract::FromRef;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub struct Link {
    pub destination: String,
    pub subdomain: Option<String>,
    pub path: Option<String>
}

type RoutableKey = (Option<String>, Option<String>); // subdomain, path
type RoutableVal = (String, String); // destination, id

#[derive(Clone)]
pub struct Collection(pub Arc<RwLock<HashMap<RoutableKey, RoutableVal>>>);

fn random_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
        abcdefghijklmnopqrstuvwxyz\
        0123456789";
        
    (0..24)
        .map(|_| CHARSET[rand::random_range(0..CHARSET.len())] as char)
        .collect()
}

impl Collection {
    // Create a new collection of redirections from a toml file
    pub fn new(path: Option<&str>) -> Self {
        let link_path = match path {
            Some(path) => path.to_string(),
            None => env::var("LINK_PATH")
                .unwrap_or("./links.toml".to_string())
        };
    
        let file = fs::read_to_string(link_path).unwrap();

        let mut col = HashMap::new();
        if let Ok(links) = toml::from_str::<HashMap<String, Link>>(&file) {
            for (id, link) in links {
                col.insert((link.subdomain, link.path), (link.destination, id));
            }
        } 

        Self(Arc::new(RwLock::new(col)))
    }

    // save to disk as toml file
    pub async fn save(&self) -> Result<(), std::io::Error> {
        let link_path = env::var("LINK_PATH")
            .unwrap_or("./links.toml".to_string());
    
        let mut links: HashMap<String, Link> = HashMap::new();
        let guard = self.0.read().await;

        for ((subdomain, path), (destination, id)) in guard.iter() {
            links.insert(id.clone(), Link {
                destination: destination.clone(),
                subdomain: subdomain.clone(),
                path: path.clone()
            });
        }

        let file = toml::to_string(&links).expect("HashMap failed to serialize");
    
        Ok(tokio::fs::write(link_path, file).await?)
    }

    pub async fn list(&self) -> HashMap<String, Link> {
        let mut map = HashMap::new();
        let guard = self.0.read().await;
        for ((subdomain, path), (destination, id)) in guard.iter() {
            map.insert(id.clone(),Link {
                subdomain: subdomain.clone(),
                destination: destination.clone(),
                path: path.clone()
            });
        }

        map
    }
    pub async fn find(&self, key: &RoutableKey) -> Option<RoutableVal> {
        let guard = self.0.read().await;
        
        // try for perfect catch
        match guard.get(key) {
            // perfect catch
            Some(value) => return Some(value.clone()),
            // confirm whether we can strip path from the search
            None if key.0.is_none() || key.1.is_none() => return None,
            _ => {}
        };
        
        // try to catch without the path just with subdomain
        guard.get(&(key.clone().0, None))
            .and_then(|value| Some(value.clone()))
    }
    
    pub async fn update_key(&mut self, key: &RoutableKey, new: RoutableKey) -> Option<()> {
        if let Some(value) = self.find(&key).await {
            let mut guard = self.0.write().await;
            guard.insert(new,value);
            guard.remove(key);
        } else {
            return None
        }
        
        self.save().await.ok()
    }

    pub async fn update_destination(&mut self, key: &RoutableKey, destination: String) -> Option<()> {
        if let Some(value) = self.find(&key).await {
            let mut guard = self.0.write().await;
            guard.insert(key.clone(), (destination, value.1));
        } else {
            return None
        }

        self.save().await.ok()
    }

    pub async fn new_pair(&mut self, key: RoutableKey, destination: String) -> Option<()> {
        let id = random_id();
    
        {
            let mut guard = self.0.write().await;
            guard.insert(key, (destination, id));
        }

        self.save().await.ok()
    }

    pub async fn remove(&mut self, key: RoutableKey) -> Option<()> {
        {
            let mut guard= self.0.write().await;
            guard.remove(&key);
        }

        self.save().await.ok()
    }
}

impl FromRef<AppState> for Collection {
    fn from_ref(input: &AppState) -> Self {
        input.collection.clone()
    }
}