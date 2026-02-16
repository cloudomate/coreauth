//! FGA Store Management
//!
//! Provides management for Fine-Grained Authorization stores, including:
//! - Store CRUD operations
//! - Authorization model management
//! - API key management for programmatic access

use crate::error::{AuthzError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Models
// ============================================================================

/// FGA Store - container for authorization models and relation tuples
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FgaStore {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub current_model_version: i32,
    pub api_key_prefix: Option<String>,
    pub is_active: bool,
    pub tuple_count: i64,
    pub settings: sqlx::types::Json<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Authorization Model - defines types and relations
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthorizationModel {
    pub id: Uuid,
    pub store_id: Uuid,
    pub version: i32,
    pub schema_json: sqlx::types::Json<AuthorizationSchema>,
    pub schema_dsl: Option<String>,
    pub is_valid: bool,
    pub validation_errors: Option<sqlx::types::Json<Vec<String>>>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Authorization Schema - OpenFGA-compatible format
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthorizationSchema {
    pub schema_version: String,
    pub type_definitions: Vec<TypeDefinition>,
}

/// Type Definition - defines an object type and its relations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub relations: std::collections::HashMap<String, RelationDefinition>,
    pub metadata: Option<TypeMetadata>,
}

/// Relation Definition - defines how a relation can be assigned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationDefinition {
    /// Direct assignment: [user, group#member]
    #[serde(default)]
    pub this: Option<DirectRelation>,
    /// Computed from another relation on the same object
    #[serde(default, alias = "computedUserset")]
    pub computed_userset: Option<ComputedUserset>,
    /// From a relation on a related object (tuple_to_userset)
    #[serde(default, alias = "tupleToUserset")]
    pub tuple_to_userset: Option<TupleToUserset>,
    /// Union of multiple relation definitions
    #[serde(default, deserialize_with = "deserialize_union_or_child")]
    pub union: Option<Vec<RelationDefinition>>,
    /// Intersection of multiple relation definitions
    #[serde(default, deserialize_with = "deserialize_union_or_child")]
    pub intersection: Option<Vec<RelationDefinition>>,
    /// Exclusion (difference) of relation definitions
    #[serde(default)]
    pub exclusion: Option<Box<Exclusion>>,
}

/// Deserialize union/intersection from either `[...]` (Rust) or `{ child: [...] }` (OpenFGA)
fn deserialize_union_or_child<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Vec<RelationDefinition>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum UnionFormat {
        Direct(Vec<RelationDefinition>),
        Wrapped { child: Vec<RelationDefinition> },
    }

    let value: Option<UnionFormat> = Option::deserialize(deserializer)?;
    Ok(value.map(|v| match v {
        UnionFormat::Direct(items) => items,
        UnionFormat::Wrapped { child } => child,
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectRelation {
    /// Allowed subject types: ["user", "group#member"]
    #[serde(default)]
    pub types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedUserset {
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TupleToUserset {
    pub tupleset: Tupleset,
    #[serde(alias = "computedUserset")]
    pub computed_userset: ComputedUserset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tupleset {
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exclusion {
    pub base: RelationDefinition,
    pub subtract: RelationDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMetadata {
    pub relations: std::collections::HashMap<String, RelationMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationMetadata {
    pub directly_related_user_types: Vec<RelatedUserType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedUserType {
    #[serde(rename = "type")]
    pub type_name: String,
    pub relation: Option<String>,
    pub wildcard: Option<serde_json::Value>,
}

/// FGA Store API Key
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FgaStoreApiKey {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub permissions: Vec<String>,
    pub rate_limit_per_minute: i32,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response when creating an API key (includes the actual key)
#[derive(Debug, Clone, Serialize)]
pub struct FgaStoreApiKeyWithSecret {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub key: String,  // Only returned once on creation
    pub key_prefix: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStoreRequest {
    pub name: String,
    pub description: Option<String>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStoreRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteModelRequest {
    pub schema: AuthorizationSchema,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Option<Vec<String>>,
    pub rate_limit_per_minute: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ============================================================================
// FGA Store Service
// ============================================================================

#[derive(Clone)]
pub struct FgaStoreService {
    pool: PgPool,
}

impl FgaStoreService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ------------------------------------------------------------------------
    // Store CRUD
    // ------------------------------------------------------------------------

    /// Create a new FGA store
    pub async fn create_store(&self, tenant_id: Uuid, request: CreateStoreRequest) -> Result<FgaStore> {
        let settings = request.settings.unwrap_or(serde_json::json!({}));

        let store = sqlx::query_as::<_, FgaStore>(
            r#"
            INSERT INTO fga_stores (tenant_id, name, description, settings)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(tenant_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(sqlx::types::Json(&settings))
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Created FGA store: {} ({})", store.name, store.id);
        Ok(store)
    }

    /// Get store by ID
    pub async fn get_store(&self, store_id: Uuid) -> Result<FgaStore> {
        sqlx::query_as::<_, FgaStore>("SELECT * FROM fga_stores WHERE id = $1")
            .bind(store_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AuthzError::NotFound(format!("Store {} not found", store_id)))
    }

    /// List all stores for a tenant
    pub async fn list_stores(&self, tenant_id: Uuid, include_inactive: bool) -> Result<Vec<FgaStore>> {
        let query = if include_inactive {
            "SELECT * FROM fga_stores WHERE tenant_id = $1 ORDER BY created_at DESC"
        } else {
            "SELECT * FROM fga_stores WHERE tenant_id = $1 AND is_active = true ORDER BY created_at DESC"
        };

        Ok(sqlx::query_as::<_, FgaStore>(query)
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update store
    pub async fn update_store(&self, store_id: Uuid, request: UpdateStoreRequest) -> Result<FgaStore> {
        let current = self.get_store(store_id).await?;

        let name = request.name.unwrap_or(current.name);
        let description = request.description.or(current.description);
        let is_active = request.is_active.unwrap_or(current.is_active);
        let settings = request.settings.unwrap_or(current.settings.0);

        let store = sqlx::query_as::<_, FgaStore>(
            r#"
            UPDATE fga_stores
            SET name = $1, description = $2, is_active = $3, settings = $4, updated_at = NOW()
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(&name)
        .bind(&description)
        .bind(is_active)
        .bind(sqlx::types::Json(&settings))
        .bind(store_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(store)
    }

    /// Delete store (soft delete by deactivating, or hard delete)
    pub async fn delete_store(&self, store_id: Uuid, hard_delete: bool) -> Result<()> {
        if hard_delete {
            sqlx::query("DELETE FROM fga_stores WHERE id = $1")
                .bind(store_id)
                .execute(&self.pool)
                .await?;
            tracing::info!("Hard deleted FGA store: {}", store_id);
        } else {
            sqlx::query("UPDATE fga_stores SET is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(store_id)
                .execute(&self.pool)
                .await?;
            tracing::info!("Soft deleted FGA store: {}", store_id);
        }
        Ok(())
    }

    // ------------------------------------------------------------------------
    // Authorization Model Management
    // ------------------------------------------------------------------------

    /// Write a new authorization model (creates new version)
    pub async fn write_model(&self, store_id: Uuid, request: WriteModelRequest) -> Result<AuthorizationModel> {
        // Validate the schema
        let validation_errors = self.validate_schema(&request.schema);
        let is_valid = validation_errors.is_empty();

        // Get next version number
        let next_version: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM fga_authorization_models WHERE store_id = $1"
        )
        .bind(store_id)
        .fetch_one(&self.pool)
        .await?;

        // Convert schema to DSL for display
        let schema_dsl = self.schema_to_dsl(&request.schema);

        // Insert the model
        let model = sqlx::query_as::<_, AuthorizationModel>(
            r#"
            INSERT INTO fga_authorization_models
                (store_id, version, schema_json, schema_dsl, is_valid, validation_errors, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(store_id)
        .bind(next_version)
        .bind(sqlx::types::Json(&request.schema))
        .bind(&schema_dsl)
        .bind(is_valid)
        .bind(if validation_errors.is_empty() {
            None
        } else {
            Some(sqlx::types::Json(&validation_errors))
        })
        .bind(&request.created_by)
        .fetch_one(&self.pool)
        .await?;

        // If valid, update the store's current model version and type definitions
        if is_valid {
            sqlx::query(
                "UPDATE fga_stores SET current_model_version = $1, updated_at = NOW() WHERE id = $2"
            )
            .bind(next_version)
            .bind(store_id)
            .execute(&self.pool)
            .await?;

            // Update type definitions
            self.update_type_definitions(store_id, next_version, &request.schema).await?;
        }

        tracing::info!(
            "Created authorization model v{} for store {} (valid: {})",
            next_version,
            store_id,
            is_valid
        );

        Ok(model)
    }

    /// Get the current authorization model for a store
    pub async fn get_current_model(&self, store_id: Uuid) -> Result<AuthorizationModel> {
        sqlx::query_as::<_, AuthorizationModel>(
            r#"
            SELECT am.*
            FROM fga_authorization_models am
            JOIN fga_stores s ON s.id = am.store_id
            WHERE am.store_id = $1 AND am.version = s.current_model_version
            "#,
        )
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthzError::NotFound("No authorization model found".to_string()))
    }

    /// Get a specific model version
    pub async fn get_model_version(&self, store_id: Uuid, version: i32) -> Result<AuthorizationModel> {
        sqlx::query_as::<_, AuthorizationModel>(
            "SELECT * FROM fga_authorization_models WHERE store_id = $1 AND version = $2"
        )
        .bind(store_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthzError::NotFound(format!("Model version {} not found", version)))
    }

    /// List all model versions for a store
    pub async fn list_models(&self, store_id: Uuid) -> Result<Vec<AuthorizationModel>> {
        Ok(sqlx::query_as::<_, AuthorizationModel>(
            "SELECT * FROM fga_authorization_models WHERE store_id = $1 ORDER BY version DESC"
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?)
    }

    // ------------------------------------------------------------------------
    // API Key Management
    // ------------------------------------------------------------------------

    /// Create an API key for a store
    pub async fn create_api_key(&self, store_id: Uuid, request: CreateApiKeyRequest) -> Result<FgaStoreApiKeyWithSecret> {
        // Generate a secure random key
        let key = format!("fga_{}", Self::generate_random_key(32));
        let key_prefix = key[..12].to_string();
        let key_hash = Self::hash_key(&key);

        let permissions = request.permissions.unwrap_or_else(|| vec![
            "read".to_string(),
            "write".to_string(),
            "check".to_string(),
        ]);
        let rate_limit = request.rate_limit_per_minute.unwrap_or(1000);

        let api_key = sqlx::query_as::<_, FgaStoreApiKey>(
            r#"
            INSERT INTO fga_store_api_keys
                (store_id, name, key_hash, key_prefix, permissions, rate_limit_per_minute, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(store_id)
        .bind(&request.name)
        .bind(&key_hash)
        .bind(&key_prefix)
        .bind(&permissions)
        .bind(rate_limit)
        .bind(request.expires_at)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Created API key '{}' for store {}", request.name, store_id);

        Ok(FgaStoreApiKeyWithSecret {
            id: api_key.id,
            store_id: api_key.store_id,
            name: api_key.name,
            key,  // Return the actual key only once
            key_prefix: api_key.key_prefix,
            permissions: api_key.permissions,
            created_at: api_key.created_at,
        })
    }

    /// List API keys for a store (without the actual key)
    pub async fn list_api_keys(&self, store_id: Uuid) -> Result<Vec<FgaStoreApiKey>> {
        Ok(sqlx::query_as::<_, FgaStoreApiKey>(
            "SELECT * FROM fga_store_api_keys WHERE store_id = $1 ORDER BY created_at DESC"
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?)
    }

    /// Revoke an API key
    pub async fn revoke_api_key(&self, key_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "UPDATE fga_store_api_keys SET is_active = false, updated_at = NOW() WHERE id = $1"
        )
        .bind(key_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AuthzError::NotFound("API key not found".to_string()));
        }

        tracing::info!("Revoked API key: {}", key_id);
        Ok(())
    }

    /// Validate an API key and return the associated store
    pub async fn validate_api_key(&self, key: &str) -> Result<(FgaStore, FgaStoreApiKey)> {
        let key_hash = Self::hash_key(key);

        let api_key = sqlx::query_as::<_, FgaStoreApiKey>(
            r#"
            SELECT * FROM fga_store_api_keys
            WHERE key_hash = $1 AND is_active = true
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(&key_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthzError::Unauthorized("Invalid or expired API key".to_string()))?;

        // Update last_used_at
        sqlx::query("UPDATE fga_store_api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(api_key.id)
            .execute(&self.pool)
            .await?;

        let store = self.get_store(api_key.store_id).await?;

        Ok((store, api_key))
    }

    // ------------------------------------------------------------------------
    // Helper Methods
    // ------------------------------------------------------------------------

    /// Validate an authorization schema
    fn validate_schema(&self, schema: &AuthorizationSchema) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for duplicate type names
        let mut type_names = std::collections::HashSet::new();
        for type_def in &schema.type_definitions {
            if !type_names.insert(&type_def.type_name) {
                errors.push(format!("Duplicate type name: {}", type_def.type_name));
            }

            // Validate relation definitions
            for (rel_name, _rel_def) in &type_def.relations {
                if rel_name.is_empty() {
                    errors.push(format!(
                        "Empty relation name in type {}",
                        type_def.type_name
                    ));
                }
            }
        }

        // TODO: Add more validation:
        // - Validate that referenced types exist
        // - Validate that referenced relations exist
        // - Check for circular dependencies

        errors
    }

    /// Convert schema to human-readable DSL
    fn schema_to_dsl(&self, schema: &AuthorizationSchema) -> String {
        let mut dsl = String::new();
        dsl.push_str(&format!("model\n  schema {}\n\n", schema.schema_version));

        for type_def in &schema.type_definitions {
            dsl.push_str(&format!("type {}\n", type_def.type_name));
            dsl.push_str("  relations\n");

            for (rel_name, rel_def) in &type_def.relations {
                dsl.push_str(&format!("    define {}: ", rel_name));
                dsl.push_str(&self.relation_def_to_dsl(rel_def));
                dsl.push('\n');
            }
            dsl.push('\n');
        }

        dsl
    }

    fn relation_def_to_dsl(&self, rel_def: &RelationDefinition) -> String {
        if let Some(direct) = &rel_def.this {
            return format!("[{}]", direct.types.join(", "));
        }

        if let Some(computed) = &rel_def.computed_userset {
            return computed.relation.clone();
        }

        if let Some(ttu) = &rel_def.tuple_to_userset {
            return format!(
                "{} from {}",
                ttu.computed_userset.relation, ttu.tupleset.relation
            );
        }

        if let Some(union) = &rel_def.union {
            let parts: Vec<String> = union.iter().map(|r| self.relation_def_to_dsl(r)).collect();
            return parts.join(" or ");
        }

        if let Some(intersection) = &rel_def.intersection {
            let parts: Vec<String> = intersection
                .iter()
                .map(|r| self.relation_def_to_dsl(r))
                .collect();
            return parts.join(" and ");
        }

        "unknown".to_string()
    }

    /// Update type definitions table for quick validation
    async fn update_type_definitions(
        &self,
        store_id: Uuid,
        version: i32,
        schema: &AuthorizationSchema,
    ) -> Result<()> {
        // Delete old definitions for this version
        sqlx::query(
            "DELETE FROM fga_type_definitions WHERE store_id = $1 AND model_version = $2"
        )
        .bind(store_id)
        .bind(version)
        .execute(&self.pool)
        .await?;

        // Insert new definitions
        for type_def in &schema.type_definitions {
            sqlx::query(
                r#"
                INSERT INTO fga_type_definitions (store_id, model_version, type_name, relations)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(store_id)
            .bind(version)
            .bind(&type_def.type_name)
            .bind(sqlx::types::Json(&type_def.relations))
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Generate a random key
    fn generate_random_key(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Hash an API key using SHA256
    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
