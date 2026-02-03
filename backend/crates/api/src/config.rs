use ciam_cache::CacheConfig;
use ciam_database::DatabaseConfig;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            database: DatabaseConfig::from_env(),
            cache: CacheConfig::from_env(),
            jwt_secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
        }
    }
}
