use std::{collections::HashMap, env, fs};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Link {
    pub destination: String,
    pub subdomain: Option<String>,
    pub path: Option<String>
}

type RoutableKey = (Option<String>, Option<String>); // subdomain, path
type RoutableVal = (String, String); // destination, id
pub struct Collection(HashMap<RoutableKey, RoutableVal>);

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

        Self(col)
    }
    pub fn save(&self) -> Result<(), std::io::Error> {
        let link_path = env::var("LINK_PATH")
            .unwrap_or("./links.toml".to_string());
    
        let mut links: HashMap<String, Link> = HashMap::new();

        for ((subdomain, path), (destination, id)) in self.0.iter() {
            links.insert(id.clone(), Link {
                destination: destination.clone(),
                subdomain: subdomain.clone(),
                path: path.clone()
            });
        }

        let file = toml::to_string(&links).expect("HashMap failed to serialize");
    
        fs::write(link_path, file)
    }
    pub fn find(&self, key: &RoutableKey) -> Option<&String> {
        match self.0.get(key) {
            Some((dest, _id)) => Some(dest),
            None => None
        }
    }
    pub fn find_id(&self, key: &RoutableKey) -> Option<&String> {
        match self.0.get(key) {
            Some((_dest, id)) => Some(id),
            None => None
        }
    }
    pub fn update_key(&mut self, key: &RoutableKey, new: RoutableKey) -> Result<(), ()> {
        let value = self.0.get(key);

        if value.is_none() {
            return Err(());
        };

        self.0.insert(new,value.unwrap().clone());
        self.0.remove(key);
        
        Ok(())
    }
    pub fn update_destination(&mut self, key: RoutableKey, destination: String) -> Result<(), ()> {
        let value = self.0.get(&key);

        if value.is_none() {
            return Err(());
        };

        self.0.insert(key, (destination, value.unwrap().1.clone()));
        Ok(())
    }
    pub fn new_path(&mut self, path: String, destination: String) {
        let id = random_id();
    
        self.0.insert((None, Some(path)), (destination, id));
    }
    pub fn new_subdomain(&mut self, subdomain: String, destination: String) {
        let id = random_id();
    
        self.0.insert((Some(subdomain), None), (destination, id));
    }
    pub fn new_subdomain_with_path(&mut self, subdomain: String, path: String, destination: String) {
        let id = random_id();
    
        self.0.insert((Some(subdomain), Some(path)), (destination, id));
    }
}