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

#[allow(unused)]
impl Collection {
    // Create a new collection of redirections from a toml file
    pub fn new(path: Option<&str>) -> Self {
        let link_path = match path {
            Some(path) => path.to_string(),
            None => env::var("LINK_PATH")
                .unwrap_or("./links.toml".to_string())
        };
    
        let file = fs::read_to_string(link_path).unwrap();
        let links: HashMap<String, Link> = match toml::from_str(&file) {
            Ok(h) => h,
            Err(_e) => HashMap::new()
        };

        let mut col = HashMap::new();
        for (id, link) in links {
            col.insert((link.subdomain, link.path), (link.destination, id));
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
    pub async fn find(&self, key: &RoutableKey) -> Option<String> {
        let guard = self.0.read().await;
        
        // try for perfect catch
        match guard.get(key) {
            // perfect catch
            Some(value) => return Some(value.0.to_string()),
            // confirm whether we can strip path from the search
            None if key.0.is_none() || key.1.is_none() => return None,
            _ => {}
        };
        
        // try to catch without the path just with subdomain
        guard.get(&(key.clone().0, None))
            .and_then(|value| Some(value.0.to_string()))
    }
    
    pub async fn find_id(&self, key: &RoutableKey) -> Option<String> {
        let guard = self.0.read().await;

        guard.get(key).and_then(|value| Some(value.1.to_string()))
    }

    pub async fn update_key(&mut self, key: &RoutableKey, new: RoutableKey) -> Result<(), ()> {
        let guard = self.0.read().await;
        let value = guard.get(key);

        if value.is_none() {
            return Err(());
        };

        let mut guard = self.0.write().await;
        guard.insert(new,value.unwrap().clone());
        guard.remove(key);
        
        drop(guard);
        self.save().await.map_err(|_| ())
    }

    pub async fn update_destination(&mut self, key: &RoutableKey, destination: String) -> Result<(), ()> {
        let guard = self.0.read().await;
        let value = guard.get(&key);

        if value.is_none() {
            return Err(());
        };

        let mut guard = self.0.write().await;
        guard.insert(key.clone(), (destination, value.unwrap().1.clone()));
        guard.remove(key);

        drop(guard);
        self.save().await.map_err(|_| ())
    }

    pub async fn new_path(&mut self, path: String, destination: String) {
        let id = random_id();
    
        let mut guard = self.0.write().await;
        guard.insert((None, Some(path)), (destination, id));
        
        drop(guard);
        self.save().await;
    }
    pub async fn new_subdomain(&mut self, subdomain: String, destination: String) {
        let id = random_id();
    
        let mut guard = self.0.write().await;
        guard.insert((Some(subdomain), None), (destination, id));
        
        drop(guard);
        self.save().await;
    }
    pub async fn new_subdomain_with_path(&mut self, subdomain: String, path: String, destination: String) {
        let id = random_id();
    
        let mut guard = self.0.write().await;
        guard.insert((Some(subdomain), Some(path)), (destination, id));
        
        drop(guard);
        self.save().await;
    }

    pub async fn remove(&mut self, key: RoutableKey) -> Result<(), ()> {
        let mut guard= self.0.write().await;

        guard.remove(&key)
            .and_then(|_| {
                self.save();
                Some(())
            })
            .ok_or(())
    }
}

impl FromRef<AppState> for Collection {
    fn from_ref(input: &AppState) -> Self {
        input.collection.clone()
    }
}