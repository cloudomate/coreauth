use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// FGA client that checks permissions via CoreAuth's FGA API.
pub struct FgaClient {
    coreauth_url: String,
    store_name: String,
    http_client: reqwest::Client,
    /// Cached store ID (resolved on first check).
    store_id: Arc<RwLock<Option<String>>>,
}

#[derive(Debug, Serialize)]
struct FgaCheckRequest {
    user: String,
    relation: String,
    object: String,
}

#[derive(Debug, Deserialize)]
struct FgaCheckResponse {
    allowed: bool,
}

#[derive(Debug, Deserialize)]
struct FgaStoreListResponse {
    stores: Vec<FgaStoreInfo>,
}

#[derive(Debug, Deserialize)]
struct FgaStoreInfo {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct FgaCreateStoreRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
struct FgaCreateStoreResponse {
    id: String,
}

impl FgaClient {
    pub fn new(coreauth_url: &str, store_name: &str, http_client: reqwest::Client) -> Self {
        Self {
            coreauth_url: coreauth_url.trim_end_matches('/').to_string(),
            store_name: store_name.to_string(),
            http_client,
            store_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if a user has a specific relation on an object.
    ///
    /// - `user_id`: The user's CoreAuth ID
    /// - `relation`: e.g., "viewer", "editor", "owner"
    /// - `object_type`: e.g., "project", "document"
    /// - `object_id`: The specific object ID
    /// - `access_token`: A valid access token for API auth
    pub async fn check_permission(
        &self,
        user_id: &str,
        relation: &str,
        object_type: &str,
        object_id: &str,
        access_token: &str,
    ) -> Result<bool, String> {
        let store_id = self.get_or_create_store(access_token).await?;

        let url = format!(
            "{}/api/fga/stores/{}/check",
            self.coreauth_url, store_id
        );

        let body = FgaCheckRequest {
            user: format!("user:{}", user_id),
            relation: relation.to_string(),
            object: format!("{}:{}", object_type, object_id),
        };

        let resp = self.http_client
            .post(&url)
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("FGA check request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("FGA check failed ({}): {}", status, body));
        }

        let result: FgaCheckResponse = resp.json().await
            .map_err(|e| format!("FGA check parse error: {}", e))?;

        Ok(result.allowed)
    }

    /// Resolve or create the FGA store by name.
    async fn get_or_create_store(&self, access_token: &str) -> Result<String, String> {
        // Check cache first
        {
            let cached = self.store_id.read().await;
            if let Some(ref id) = *cached {
                return Ok(id.clone());
            }
        }

        // Try to find existing store
        let list_url = format!("{}/api/fga/stores", self.coreauth_url);
        let resp = self.http_client
            .get(&list_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("FGA list stores failed: {}", e))?;

        if resp.status().is_success() {
            if let Ok(list) = resp.json::<FgaStoreListResponse>().await {
                if let Some(store) = list.stores.iter().find(|s| s.name == self.store_name) {
                    let id = store.id.clone();
                    tracing::info!("Found FGA store '{}' → {}", self.store_name, id);
                    let mut cached = self.store_id.write().await;
                    *cached = Some(id.clone());
                    return Ok(id);
                }
            }
        }

        // Store not found — create it
        tracing::info!("Creating FGA store '{}'", self.store_name);
        let create_url = format!("{}/api/fga/stores", self.coreauth_url);
        let resp = self.http_client
            .post(&create_url)
            .bearer_auth(access_token)
            .json(&FgaCreateStoreRequest {
                name: self.store_name.clone(),
            })
            .send()
            .await
            .map_err(|e| format!("FGA create store failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("FGA create store failed ({}): {}", status, body));
        }

        let created: FgaCreateStoreResponse = resp.json().await
            .map_err(|e| format!("FGA create store parse error: {}", e))?;

        tracing::info!("Created FGA store '{}' → {}", self.store_name, created.id);
        let mut cached = self.store_id.write().await;
        *cached = Some(created.id.clone());
        Ok(created.id)
    }
}
