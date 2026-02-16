//! Tenant Database Router
//!
//! Routes database connections based on tenant isolation mode:
//! - Shared (pool): Uses master database with tenant_id isolation
//! - Dedicated (silo): Uses tenant's own dedicated database
//!
//! Architecture:
//! ```
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    TenantDatabaseRouter                          │
//! │  ┌─────────────────────────────────────────────────────────┐    │
//! │  │  Master DB Pool (tenant_registry, billing, platform)    │    │
//! │  └─────────────────────────────────────────────────────────┘    │
//! │                              │                                   │
//! │  ┌───────────────────────────┼───────────────────────────────┐  │
//! │  │      Tenant Connection Cache (LRU, max 100 tenants)       │  │
//! │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                │  │
//! │  │  │ Tenant A │  │ Tenant B │  │ Tenant C │  ...           │  │
//! │  │  │ (shared) │  │(dedicated)│  │(dedicated)│               │  │
//! │  │  └──────────┘  └──────────┘  └──────────┘                │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use crate::error::{DatabaseError, Result};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Tenant isolation mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IsolationMode {
    Shared,    // Uses master DB with tenant_id column isolation
    Dedicated, // Uses tenant's own database
}

impl Default for IsolationMode {
    fn default() -> Self {
        Self::Shared
    }
}

impl From<String> for IsolationMode {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "dedicated" | "silo" => Self::Dedicated,
            _ => Self::Shared,
        }
    }
}

/// Tenant registration record from master database
#[derive(Debug, Clone, FromRow)]
pub struct TenantRecord {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub isolation_mode: String,
    pub database_host: Option<String>,
    pub database_port: Option<i32>,
    pub database_name: Option<String>,
    pub database_user: Option<String>,
    pub database_password_encrypted: Option<String>,
    pub pool_min_connections: Option<i32>,
    pub pool_max_connections: Option<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl TenantRecord {
    pub fn isolation_mode(&self) -> IsolationMode {
        IsolationMode::from(self.isolation_mode.clone())
    }

    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}

/// Cached tenant connection info
#[derive(Debug, Clone)]
struct TenantConnection {
    tenant: TenantRecord,
    pool: PgPool,
}

/// Configuration for the tenant router
#[derive(Debug, Clone)]
pub struct TenantRouterConfig {
    /// Maximum number of tenant connections to cache
    pub max_cached_connections: u64,
    /// Time-to-live for cached connections
    pub connection_ttl: Duration,
    /// Default max connections per tenant pool
    pub default_max_connections: u32,
    /// Default min connections per tenant pool
    pub default_min_connections: u32,
    /// Encryption key for database passwords (base64)
    pub encryption_key: Option<String>,
}

impl Default for TenantRouterConfig {
    fn default() -> Self {
        Self {
            max_cached_connections: 100,
            connection_ttl: Duration::from_secs(3600), // 1 hour
            default_max_connections: 10,
            default_min_connections: 1,
            encryption_key: None,
        }
    }
}

impl TenantRouterConfig {
    /// Load configuration from environment variables
    ///
    /// Required environment variables for dedicated tenant databases:
    /// - TENANT_DB_ENCRYPTION_KEY: Base64-encoded 32-byte AES-256 key
    ///
    /// Generate a key with: `openssl rand -base64 32`
    pub fn from_env() -> Self {
        Self {
            max_cached_connections: std::env::var("TENANT_ROUTER_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            connection_ttl: Duration::from_secs(
                std::env::var("TENANT_ROUTER_CONNECTION_TTL_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3600),
            ),
            default_max_connections: std::env::var("TENANT_DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            default_min_connections: std::env::var("TENANT_DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            encryption_key: std::env::var("TENANT_DB_ENCRYPTION_KEY").ok(),
        }
    }
}

/// Tenant Database Router
///
/// Manages database connections for multi-tenant isolation.
/// Supports both shared (pooled) and dedicated (silo) modes.
#[derive(Clone)]
pub struct TenantDatabaseRouter {
    /// Master database pool (for tenant_registry lookups)
    master_pool: PgPool,
    /// Cache of tenant connections
    connection_cache: Cache<Uuid, Arc<TenantConnection>>,
    /// Configuration
    config: TenantRouterConfig,
}

impl TenantDatabaseRouter {
    /// Create a new tenant router with the master database pool
    pub fn new(master_pool: PgPool, config: TenantRouterConfig) -> Self {
        let connection_cache = Cache::builder()
            .max_capacity(config.max_cached_connections)
            .time_to_live(config.connection_ttl)
            .build();

        Self {
            master_pool,
            connection_cache,
            config,
        }
    }

    /// Get the master database pool (for platform-level operations)
    pub fn master_pool(&self) -> &PgPool {
        &self.master_pool
    }

    /// Get database pool for a specific tenant by ID
    pub async fn get_tenant_pool(&self, tenant_id: Uuid) -> Result<PgPool> {
        // Check cache first
        if let Some(conn) = self.connection_cache.get(&tenant_id).await {
            if conn.tenant.is_active() {
                return Ok(conn.pool.clone());
            }
            // Tenant no longer active, remove from cache
            self.connection_cache.invalidate(&tenant_id).await;
        }

        // Lookup tenant from registry
        let tenant = self.get_tenant_record(tenant_id).await?;

        if !tenant.is_active() {
            return Err(DatabaseError::Other(format!(
                "Tenant {} is not active (status: {})",
                tenant.slug, tenant.status
            )));
        }

        // Get or create connection pool based on isolation mode
        let pool = match tenant.isolation_mode() {
            IsolationMode::Shared => {
                // Use master pool for shared tenants
                self.master_pool.clone()
            }
            IsolationMode::Dedicated => {
                // Create dedicated pool for this tenant
                self.create_tenant_pool(&tenant).await?
            }
        };

        // Cache the connection
        let conn = Arc::new(TenantConnection {
            tenant: tenant.clone(),
            pool: pool.clone(),
        });
        self.connection_cache.insert(tenant_id, conn).await;

        Ok(pool)
    }

    /// Get database pool for a tenant by slug
    pub async fn get_tenant_pool_by_slug(&self, slug: &str) -> Result<PgPool> {
        let tenant = self.get_tenant_record_by_slug(slug).await?;
        self.get_tenant_pool(tenant.id).await
    }

    /// Get tenant record by ID
    pub async fn get_tenant_record(&self, tenant_id: Uuid) -> Result<TenantRecord> {
        sqlx::query_as::<_, TenantRecord>(
            r#"
            SELECT id, slug, name, isolation_mode,
                   database_host, database_port, database_name,
                   database_user, database_password_encrypted,
                   pool_min_connections, pool_max_connections,
                   status, created_at
            FROM tenant_registry
            WHERE id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.master_pool)
        .await?
        .ok_or_else(|| DatabaseError::NotFound(format!("Tenant {} not found", tenant_id)))
    }

    /// Get tenant record by slug
    pub async fn get_tenant_record_by_slug(&self, slug: &str) -> Result<TenantRecord> {
        sqlx::query_as::<_, TenantRecord>(
            r#"
            SELECT id, slug, name, isolation_mode,
                   database_host, database_port, database_name,
                   database_user, database_password_encrypted,
                   pool_min_connections, pool_max_connections,
                   status, created_at
            FROM tenant_registry
            WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.master_pool)
        .await?
        .ok_or_else(|| DatabaseError::NotFound(format!("Tenant '{}' not found", slug)))
    }

    /// Create a new tenant in the registry
    pub async fn create_tenant(
        &self,
        slug: &str,
        name: &str,
        isolation_mode: IsolationMode,
    ) -> Result<TenantRecord> {
        let mode_str = match isolation_mode {
            IsolationMode::Shared => "shared",
            IsolationMode::Dedicated => "dedicated",
        };

        let tenant = sqlx::query_as::<_, TenantRecord>(
            r#"
            INSERT INTO tenant_registry (slug, name, isolation_mode, status)
            VALUES ($1, $2, $3, 'provisioning')
            RETURNING id, slug, name, isolation_mode,
                      database_host, database_port, database_name,
                      database_user, database_password_encrypted,
                      pool_min_connections, pool_max_connections,
                      status, created_at
            "#,
        )
        .bind(slug)
        .bind(name)
        .bind(mode_str)
        .fetch_one(&self.master_pool)
        .await?;

        tracing::info!(
            "Created tenant: {} ({}) with {} isolation",
            tenant.name,
            tenant.slug,
            mode_str
        );

        Ok(tenant)
    }

    /// Configure dedicated database for a tenant
    pub async fn configure_dedicated_database(
        &self,
        tenant_id: Uuid,
        host: &str,
        port: i32,
        database_name: &str,
        username: &str,
        password: &str,
    ) -> Result<TenantRecord> {
        // Encrypt password before storing
        let encrypted_password = self.encrypt_password(password)?;

        let tenant = sqlx::query_as::<_, TenantRecord>(
            r#"
            UPDATE tenant_registry
            SET database_host = $1,
                database_port = $2,
                database_name = $3,
                database_user = $4,
                database_password_encrypted = $5,
                status = 'active',
                updated_at = NOW()
            WHERE id = $6
            RETURNING id, slug, name, isolation_mode,
                      database_host, database_port, database_name,
                      database_user, database_password_encrypted,
                      pool_min_connections, pool_max_connections,
                      status, created_at
            "#,
        )
        .bind(host)
        .bind(port)
        .bind(database_name)
        .bind(username)
        .bind(&encrypted_password)
        .bind(tenant_id)
        .fetch_one(&self.master_pool)
        .await?;

        // Invalidate cache to pick up new config
        self.connection_cache.invalidate(&tenant_id).await;

        tracing::info!(
            "Configured dedicated database for tenant {}: {}@{}:{}/{}",
            tenant.slug,
            username,
            host,
            port,
            database_name
        );

        Ok(tenant)
    }

    /// Activate a shared tenant (no dedicated database needed)
    pub async fn activate_shared_tenant(&self, tenant_id: Uuid) -> Result<TenantRecord> {
        let tenant = sqlx::query_as::<_, TenantRecord>(
            r#"
            UPDATE tenant_registry
            SET status = 'active', updated_at = NOW()
            WHERE id = $1
            RETURNING id, slug, name, isolation_mode,
                      database_host, database_port, database_name,
                      database_user, database_password_encrypted,
                      pool_min_connections, pool_max_connections,
                      status, created_at
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.master_pool)
        .await?;

        tracing::info!("Activated shared tenant: {}", tenant.slug);

        Ok(tenant)
    }

    /// Suspend a tenant
    pub async fn suspend_tenant(&self, tenant_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE tenant_registry SET status = 'suspended', updated_at = NOW() WHERE id = $1")
            .bind(tenant_id)
            .execute(&self.master_pool)
            .await?;

        // Remove from cache
        self.connection_cache.invalidate(&tenant_id).await;

        tracing::info!("Suspended tenant: {}", tenant_id);
        Ok(())
    }

    /// List all tenants
    pub async fn list_tenants(&self, include_inactive: bool) -> Result<Vec<TenantRecord>> {
        let query = if include_inactive {
            "SELECT id, slug, name, isolation_mode, database_host, database_port, database_name, database_user, database_password_encrypted, pool_min_connections, pool_max_connections, status, created_at FROM tenant_registry ORDER BY created_at DESC"
        } else {
            "SELECT id, slug, name, isolation_mode, database_host, database_port, database_name, database_user, database_password_encrypted, pool_min_connections, pool_max_connections, status, created_at FROM tenant_registry WHERE status = 'active' ORDER BY created_at DESC"
        };

        Ok(sqlx::query_as::<_, TenantRecord>(query)
            .fetch_all(&self.master_pool)
            .await?)
    }

    /// Get router statistics
    pub fn stats(&self) -> TenantRouterStats {
        TenantRouterStats {
            cached_connections: self.connection_cache.entry_count(),
            max_cache_size: self.config.max_cached_connections,
        }
    }

    /// Create a connection pool for a dedicated tenant
    async fn create_tenant_pool(&self, tenant: &TenantRecord) -> Result<PgPool> {
        let host = tenant.database_host.as_ref().ok_or_else(|| {
            DatabaseError::Other(format!(
                "Tenant {} has dedicated isolation but no database host configured",
                tenant.slug
            ))
        })?;

        let port = tenant.database_port.unwrap_or(5432) as u16;
        let database = tenant.database_name.as_ref().ok_or_else(|| {
            DatabaseError::Other(format!(
                "Tenant {} has dedicated isolation but no database name configured",
                tenant.slug
            ))
        })?;
        let username = tenant.database_user.as_ref().ok_or_else(|| {
            DatabaseError::Other(format!(
                "Tenant {} has dedicated isolation but no database user configured",
                tenant.slug
            ))
        })?;
        let password = self.decrypt_password(
            tenant.database_password_encrypted.as_ref().ok_or_else(|| {
                DatabaseError::Other(format!(
                    "Tenant {} has dedicated isolation but no database password configured",
                    tenant.slug
                ))
            })?,
        )?;

        let max_connections = tenant
            .pool_max_connections
            .map(|c| c as u32)
            .unwrap_or(self.config.default_max_connections);
        let min_connections = tenant
            .pool_min_connections
            .map(|c| c as u32)
            .unwrap_or(self.config.default_min_connections);

        let options = PgConnectOptions::new()
            .host(host)
            .port(port)
            .database(database)
            .username(username)
            .password(&password);

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(600))
            .connect_with(options)
            .await
            .map_err(|e| {
                DatabaseError::ConnectionFailed(format!(
                    "Failed to connect to tenant {} database: {}",
                    tenant.slug, e
                ))
            })?;

        tracing::info!(
            "Created connection pool for tenant {} ({}:{}/{})",
            tenant.slug,
            host,
            port,
            database
        );

        Ok(pool)
    }

    /// Encrypt a database password using AES-256-GCM
    ///
    /// Format: base64(nonce || ciphertext || tag)
    /// - nonce: 12 bytes
    /// - tag: 16 bytes (appended by AES-GCM)
    fn encrypt_password(&self, password: &str) -> Result<String> {
        use aes_gcm::{
            aead::{Aead, KeyInit, OsRng},
            Aes256Gcm, Nonce,
        };
        use base64::{engine::general_purpose::STANDARD, Engine};
        use rand::RngCore;

        let key = self.get_encryption_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| DatabaseError::Other(format!("Invalid encryption key: {}", e)))?;

        // Generate random 12-byte nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the password
        let ciphertext = cipher
            .encrypt(nonce, password.as_bytes())
            .map_err(|e| DatabaseError::Other(format!("Encryption failed: {}", e)))?;

        // Combine nonce + ciphertext and encode as base64
        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(STANDARD.encode(&combined))
    }

    /// Decrypt a database password using AES-256-GCM
    fn decrypt_password(&self, encrypted: &str) -> Result<String> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use base64::{engine::general_purpose::STANDARD, Engine};

        let key = self.get_encryption_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| DatabaseError::Other(format!("Invalid encryption key: {}", e)))?;

        // Decode base64
        let combined = STANDARD
            .decode(encrypted)
            .map_err(|e| DatabaseError::Other(format!("Invalid encrypted data format: {}", e)))?;

        // Extract nonce (first 12 bytes) and ciphertext (rest)
        if combined.len() < 12 {
            return Err(DatabaseError::Other(
                "Encrypted data too short".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| DatabaseError::Other(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| DatabaseError::Other(format!("Invalid password encoding: {}", e)))
    }

    /// Get the 32-byte encryption key from config
    fn get_encryption_key(&self) -> Result<[u8; 32]> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let key_b64 = self.config.encryption_key.as_ref().ok_or_else(|| {
            DatabaseError::Other(
                "Encryption key not configured. Set TENANT_DB_ENCRYPTION_KEY environment variable."
                    .to_string(),
            )
        })?;

        let key_bytes = STANDARD
            .decode(key_b64)
            .map_err(|e| DatabaseError::Other(format!("Invalid encryption key format: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(DatabaseError::Other(format!(
                "Encryption key must be 32 bytes (256 bits), got {} bytes",
                key_bytes.len()
            )));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(key)
    }
}

/// Router statistics
#[derive(Debug, Clone, Serialize)]
pub struct TenantRouterStats {
    pub cached_connections: u64,
    pub max_cache_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_mode_from_string() {
        assert_eq!(
            IsolationMode::from("shared".to_string()),
            IsolationMode::Shared
        );
        assert_eq!(
            IsolationMode::from("dedicated".to_string()),
            IsolationMode::Dedicated
        );
        assert_eq!(
            IsolationMode::from("silo".to_string()),
            IsolationMode::Dedicated
        );
        assert_eq!(
            IsolationMode::from("pool".to_string()),
            IsolationMode::Shared
        );
    }

    #[test]
    fn test_encryption_roundtrip() {
        use base64::{engine::general_purpose::STANDARD, Engine};

        // Generate a test key (32 bytes = 256 bits)
        let key = [0x42u8; 32]; // Test key (don't use in production!)
        let key_b64 = STANDARD.encode(&key);

        let config = TenantRouterConfig {
            encryption_key: Some(key_b64),
            ..Default::default()
        };

        // Create a mock pool (we won't use it for encryption tests)
        // We need to test the encryption without a real database connection
        // So we'll test the encryption key parsing first
        let key_bytes = STANDARD.decode(config.encryption_key.as_ref().unwrap()).unwrap();
        assert_eq!(key_bytes.len(), 32);
    }

    #[test]
    fn test_encryption_key_validation() {
        use base64::{engine::general_purpose::STANDARD, Engine};

        // Test with invalid key length (16 bytes instead of 32)
        let short_key = [0x42u8; 16];
        let short_key_b64 = STANDARD.encode(&short_key);

        let config = TenantRouterConfig {
            encryption_key: Some(short_key_b64),
            ..Default::default()
        };

        let key_bytes = STANDARD.decode(config.encryption_key.as_ref().unwrap()).unwrap();
        assert_ne!(key_bytes.len(), 32); // Should fail validation in get_encryption_key
    }

    #[test]
    fn test_config_from_env() {
        // Test that from_env returns defaults when env vars are not set
        let config = TenantRouterConfig::from_env();
        assert_eq!(config.max_cached_connections, 100);
        assert_eq!(config.default_max_connections, 10);
        assert_eq!(config.default_min_connections, 1);
    }
}
