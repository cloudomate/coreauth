use crate::error::Result;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub url: String,
    pub pool_size: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
        }
    }
}

impl CacheConfig {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("REDIS_URL").unwrap_or_else(|_| Self::default().url),
            pool_size: std::env::var("REDIS_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        }
    }
}

#[derive(Clone)]
pub struct Cache {
    manager: ConnectionManager,
}

impl Cache {
    pub async fn new(config: CacheConfig) -> Result<Self> {
        let client = Client::open(config.url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(Self { manager })
    }

    /// Set a value in the cache with optional TTL (seconds)
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: Option<usize>,
    ) -> Result<()> {
        let serialized = serde_json::to_string(value)?;
        let mut conn = self.manager.clone();

        if let Some(ttl) = ttl_seconds {
            conn.set_ex::<_, _, ()>(key, serialized, ttl as u64).await?;
        } else {
            conn.set::<_, _, ()>(key, serialized).await?;
        }

        Ok(())
    }

    /// Get a value from the cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.manager.clone();
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(s) => {
                let deserialized = serde_json::from_str(&s)?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Delete a key from the cache
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    /// Delete multiple keys matching a pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64> {
        let mut conn = self.manager.clone();
        let keys: Vec<String> = conn.keys(pattern).await?;

        if keys.is_empty() {
            return Ok(0);
        }

        let count = keys.len() as u64;
        conn.del::<_, ()>(keys).await?;
        Ok(count)
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.manager.clone();
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    /// Set expiration on a key (seconds)
    pub async fn expire(&self, key: &str, seconds: usize) -> Result<()> {
        let mut conn = self.manager.clone();
        conn.expire::<_, ()>(key, seconds as i64).await?;
        Ok(())
    }

    /// Get TTL of a key (seconds remaining)
    pub async fn ttl(&self, key: &str) -> Result<i64> {
        let mut conn = self.manager.clone();
        let ttl: i64 = conn.ttl(key).await?;
        Ok(ttl)
    }

    /// Increment a counter
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.manager.clone();
        let value: i64 = conn.incr(key, 1).await?;
        Ok(value)
    }

    /// Increment a counter with TTL
    pub async fn incr_with_ttl(&self, key: &str, ttl_seconds: usize) -> Result<i64> {
        let mut conn = self.manager.clone();
        let value: i64 = conn.incr(key, 1).await?;
        conn.expire::<_, ()>(key, ttl_seconds as i64).await?;
        Ok(value)
    }

    /// Get multiple values at once
    pub async fn mget<T: DeserializeOwned>(&self, keys: &[String]) -> Result<Vec<Option<T>>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self.manager.clone();
        let values: Vec<Option<String>> = conn.get(keys).await?;

        let mut results = Vec::with_capacity(values.len());
        for value in values {
            match value {
                Some(s) => {
                    let deserialized = serde_json::from_str(&s)?;
                    results.push(Some(deserialized));
                }
                None => results.push(None),
            }
        }

        Ok(results)
    }

    /// Set multiple values at once
    pub async fn mset<T: Serialize>(&self, items: &[(String, T)]) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        let mut conn = self.manager.clone();
        let mut pairs: Vec<(String, String)> = Vec::with_capacity(items.len());

        for (key, value) in items {
            let serialized = serde_json::to_string(value)?;
            pairs.push((key.clone(), serialized));
        }

        conn.mset::<_, _, ()>(&pairs).await?;
        Ok(())
    }

    /// Ping Redis to check connection
    pub async fn ping(&self) -> Result<()> {
        let mut conn = self.manager.clone();
        redis::cmd("PING").query_async::<()>(&mut conn).await?;
        Ok(())
    }

    /// Flush all keys (WARNING: Use with caution!)
    pub async fn flush_all(&self) -> Result<()> {
        let mut conn = self.manager.clone();
        redis::cmd("FLUSHALL").query_async::<()>(&mut conn).await?;
        Ok(())
    }
}

// Helper functions for common cache key patterns
pub fn user_cache_key(user_id: &str) -> String {
    format!("user:{}", user_id)
}

pub fn tenant_cache_key(tenant_id: &str) -> String {
    format!("tenant:{}", tenant_id)
}

pub fn session_cache_key(token_hash: &str) -> String {
    format!("session:{}", token_hash)
}

pub fn rate_limit_key(identifier: &str, window: &str) -> String {
    format!("ratelimit:{}:{}", identifier, window)
}

pub fn authz_cache_key(user_id: &str, resource: &str, action: &str) -> String {
    format!("authz:{}:{}:{}", user_id, resource, action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with Redis available
    async fn test_redis_connection() {
        let config = CacheConfig::from_env();
        let cache = Cache::new(config).await.expect("Failed to connect to Redis");
        cache.ping().await.expect("Failed to ping Redis");
    }

    #[tokio::test]
    #[ignore]
    async fn test_set_get() {
        let config = CacheConfig::from_env();
        let cache = Cache::new(config).await.unwrap();

        cache.set("test_key", &"test_value", Some(60)).await.unwrap();
        let value: Option<String> = cache.get("test_key").await.unwrap();

        assert_eq!(value, Some("test_value".to_string()));
        cache.delete("test_key").await.unwrap();
    }
}
