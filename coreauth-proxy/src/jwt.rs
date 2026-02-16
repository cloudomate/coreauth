use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// JWT claims extracted from CoreAuth-issued tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub organization_id: Option<String>,
    #[serde(default)]
    pub organization_slug: Option<String>,
    #[serde(alias = "org_id")]
    pub org_id: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub is_platform_admin: Option<bool>,
    #[serde(default)]
    pub scope: Option<String>,
    pub exp: i64,
    #[serde(default)]
    pub iat: Option<i64>,
    #[serde(default)]
    pub iss: Option<String>,
    #[serde(default)]
    pub aud: Option<serde_json::Value>,
    #[serde(default)]
    pub azp: Option<String>,
}

/// JWKS key entry.
#[derive(Debug, Clone, Deserialize)]
pub struct JwkKey {
    pub kid: Option<String>,
    pub kty: String,
    #[serde(default)]
    pub alg: Option<String>,
    #[serde(rename = "use")]
    #[allow(dead_code)]
    pub key_use: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

/// JWT validator that fetches and caches JWKS from CoreAuth.
pub struct JwtValidator {
    coreauth_url: String,
    /// Cached decoding keys by kid
    keys: Arc<RwLock<HashMap<String, (DecodingKey, Algorithm)>>>,
    http_client: reqwest::Client,
}

impl JwtValidator {
    pub fn new(coreauth_url: &str) -> Self {
        Self {
            coreauth_url: coreauth_url.trim_end_matches('/').to_string(),
            keys: Arc::new(RwLock::new(HashMap::new())),
            http_client: reqwest::Client::new(),
        }
    }

    /// Fetch JWKS from CoreAuth and cache the keys.
    pub async fn refresh_jwks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/.well-known/jwks.json", self.coreauth_url);
        tracing::info!("Fetching JWKS from {}", url);

        let resp: JwksResponse = self.http_client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        let mut keys = self.keys.write().await;
        keys.clear();

        for key in resp.keys {
            if key.kty != "RSA" {
                continue;
            }

            let kid = match &key.kid {
                Some(kid) => kid.clone(),
                None => continue,
            };

            let (n, e) = match (&key.n, &key.e) {
                (Some(n), Some(e)) => (n.clone(), e.clone()),
                _ => continue,
            };

            let alg = match key.alg.as_deref() {
                Some("RS256") | None => Algorithm::RS256,
                Some("RS384") => Algorithm::RS384,
                Some("RS512") => Algorithm::RS512,
                _ => continue,
            };

            match DecodingKey::from_rsa_components(&n, &e) {
                Ok(decoding_key) => {
                    tracing::info!("Cached JWKS key: kid={}", kid);
                    keys.insert(kid, (decoding_key, alg));
                }
                Err(e) => {
                    tracing::warn!("Failed to parse JWKS key: {}", e);
                }
            }
        }

        tracing::info!("JWKS cache refreshed: {} keys", keys.len());
        Ok(())
    }

    /// Validate a JWT token and return the claims.
    pub async fn validate(&self, token: &str) -> Result<Claims, String> {
        // Decode header to get kid
        let header = decode_header(token)
            .map_err(|e| format!("Invalid JWT header: {}", e))?;

        let kid = header.kid
            .ok_or_else(|| "JWT missing kid header".to_string())?;

        // Look up key
        let keys = self.keys.read().await;
        let (decoding_key, algorithm) = keys.get(&kid)
            .ok_or_else(|| format!("Unknown JWT kid: {}", kid))?;

        // Validate
        let mut validation = Validation::new(*algorithm);
        validation.validate_exp = true;
        // Don't validate audience — the proxy accepts any token issued by CoreAuth
        validation.validate_aud = false;

        let token_data = decode::<Claims>(token, decoding_key, &validation)
            .map_err(|e| format!("JWT validation failed: {}", e))?;

        Ok(token_data.claims)
    }

    /// Start a background task to periodically refresh JWKS.
    pub fn start_refresh_task(self: &Arc<Self>, interval_secs: u64) {
        let validator = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(interval_secs),
            );
            loop {
                interval.tick().await;
                if let Err(e) = validator.refresh_jwks().await {
                    tracing::warn!("JWKS refresh failed: {}", e);
                }
            }
        });
    }
}
