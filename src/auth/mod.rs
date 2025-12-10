// mod surreal;
mod http_surreal;

pub trait AuthDriver {
    async fn login(user: String, pass: String) -> Result<String, String>;
    async fn authentificate(token: String) -> Result<(), String>;
}

pub type Auth = http_surreal::SurrealDB;