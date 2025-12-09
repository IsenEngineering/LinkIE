use serde::Serialize;
use std::env;
use super::AuthDriver;

pub struct SurrealDB;

#[derive(Serialize)]
struct Credentials {
    email: String,
    password: String
}

use surrealdb::{Surreal, engine::remote::http::{Client, Http}, opt::auth::Record};
impl SurrealDB {
    async fn db() -> Result<Surreal<Client>, String> {
        let endpoint = env::var("SURREAL_ENDPOINT")
            .unwrap_or("tide-db.isenengineering.fr".to_string());

        match Surreal::new::<Http>(endpoint).await {
            Ok(db) => Ok(db),
            Err(e) => return Err(e.to_string())
        }
    }
}

impl AuthDriver for SurrealDB {
    async fn authentificate(token: String) -> Result<(), String> {
        let db = SurrealDB::db().await?;
        if db.authenticate(token).await.is_err() {
            return Err("Jeton invalid".to_string())
        }

        let query = "fn::permissions($session.rd).any(permission:linkie)";
        let resp: Option<bool> = match db.query(query).await {
            Err(e) => return Err(e.to_string()),
            Ok(mut resp) => resp.take(0).unwrap()
        };

        match resp.unwrap() {
            true => Ok(()),
            false => Err("Permissions insuffisantes".to_string())
        }
    }
    async fn login(user: String, pass: String) -> Result<String, String> {
        let db = SurrealDB::db().await?;

        let jwt = db.signin(Record {
            namespace: &env::var("SURREAL_NS").unwrap_or("tidee".to_string()),
            database: &env::var("SURREAL_DB").unwrap_or("data".to_string()),
            access: &env::var("SURREAL_AC").unwrap_or("membres".to_string()),
            params: Credentials {
                email: user,
                password: pass
            },
        }).await;

        match jwt {
            Ok(jwt) => Ok(jwt.as_insecure_token().to_string()),
            Err(e) => Err(e.to_string())
        }
    }
}