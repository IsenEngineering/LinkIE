use serde::{Deserialize, Serialize};
use std::env;
use super::AuthDriver;

pub struct SurrealDB;

impl SurrealDB {
    fn namespace() -> String {
        env::var("SURREAL_NS").unwrap_or("tidee".to_string())
    }
    fn database() -> String {
        env::var("SURREAL_DB").unwrap_or("data".to_string())
    }
    fn endpoint(path: Option<String>) -> String {
        let endpoint = env::var("SURREAL_ENDPOINT")
            .unwrap_or("http://tide-db.isenengineering.fr".to_string());

        match path {
            None => endpoint,
            Some(path) => format!("{}/{}", endpoint, path.trim_start_matches('/'))
        }
    }
}

#[derive(Deserialize)]
struct SurrealSqlResponse {
    result: bool
}

#[derive(Serialize)]
struct SurrealSigninRecord {
    pub ns: String,
    pub db: String,
    pub ac: String,
    pub email: String,
    pub password: String 
}
#[derive(Deserialize)]
struct SurrealSigninResponse {
    code: u32,
    token: String
}

impl AuthDriver for SurrealDB {
    async fn authentificate(token: String) -> Result<(), String> {
        let client = reqwest::Client::new();
        let endpoint = SurrealDB::endpoint(Some("/sql".to_string()));

        let resp = client
            .post(endpoint)
            .body("fn::permissions($session.rd).any(permissions:linkie)")
            .header("Accept", "application/json")
            .header("ns", SurrealDB::namespace())
            .header("db", SurrealDB::database())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err("Forbidden".to_string())
        }
        let text = resp.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let json: Vec<SurrealSqlResponse> = serde_json::from_str(&text)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        json.first()
            .ok_or("Empty response".to_string())?
            .result
            .then_some(())
            .ok_or("Authentication failure".to_string())
    }
    async fn login(user: String, pass: String) -> Result<String, String> {
        let client = reqwest::Client::new();
        let endpoint = SurrealDB::endpoint(Some("/signin".to_string()));

        let record = SurrealSigninRecord {
            email: user,
            password: pass,
            ns: SurrealDB::namespace(),
            db: SurrealDB::database(),
            ac: "membres".to_string()
        };

        let resp = client
            .post(endpoint)
            .body(serde_json::to_string(&record).unwrap())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err("Bad Request".to_string());
        }

        let text = resp.bytes().await
            .map_err(|e| format!("Failed to read response: {}", e))?;
        let json: SurrealSigninResponse = serde_json::from_slice(&text)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        match json.code {
            200 => Ok(json.token),
            _ => Err(format!("Authentication failure (code: {})", json.code))
        }
    }
}