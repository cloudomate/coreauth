use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use axum::http::header::COOKIE;
use axum::http::HeaderMap;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::SessionConfig;

/// Data stored in the server-side session store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub email: String,
    pub tenant_id: Option<String>,
    pub tenant_slug: Option<String>,
    pub role: Option<String>,
    pub is_platform_admin: bool,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    /// Unix timestamp when the access token expires.
    pub expires_at: i64,
}

/// Manages server-side sessions with a small encrypted cookie holding just the session ID.
///
/// Previously the entire SessionData (including JWTs) was encrypted into the cookie,
/// which exceeded the browser's 4KB cookie limit. Now the cookie only contains an
/// encrypted session ID (~100 bytes), and the full data lives in memory.
pub struct SessionManager {
    cipher: Aes256Gcm,
    store: Arc<RwLock<HashMap<String, SessionData>>>,
    cookie_name: String,
    cookie_domain: String,
    max_age_seconds: u64,
    secure: bool,
}

impl SessionManager {
    pub fn new(config: &SessionConfig) -> Self {
        // Derive a 256-bit key from the secret using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(config.secret.as_bytes());
        let key_bytes = hasher.finalize();
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .expect("AES-256-GCM key must be 32 bytes");

        Self {
            cipher,
            store: Arc::new(RwLock::new(HashMap::new())),
            cookie_name: config.cookie_name.clone(),
            cookie_domain: config.cookie_domain.clone(),
            max_age_seconds: config.max_age_seconds,
            secure: config.secure,
        }
    }

    /// Extract and decrypt the session ID from request cookies (synchronous).
    pub fn extract_session_id(&self, headers: &HeaderMap) -> Option<String> {
        let cookie_header = headers.get(COOKIE)?.to_str().ok()?;
        let prefix = format!("{}=", self.cookie_name);
        let encoded = cookie_header
            .split(';')
            .map(|s| s.trim())
            .find(|s| s.starts_with(&prefix))?
            .strip_prefix(&prefix)?;

        let payload = URL_SAFE_NO_PAD.decode(encoded).ok()?;
        if payload.len() < 12 {
            return None;
        }

        let (nonce_bytes, ciphertext) = payload.split_at(12);
        let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext).ok()?;
        String::from_utf8(plaintext).ok()
    }

    /// Look up session data by session ID.
    pub async fn get_session(&self, session_id: &str) -> Option<SessionData> {
        self.store.read().await.get(session_id).cloned()
    }

    /// Create a new session: store the data and return a Set-Cookie header value.
    pub async fn create_session(&self, data: SessionData) -> Result<String, String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        tracing::debug!("Created session {} for user {}", session_id, data.user_id);
        self.store.write().await.insert(session_id.clone(), data);
        self.encrypt_cookie(&session_id)
    }

    /// Update an existing session's data in the store.
    pub async fn update_session(&self, session_id: &str, data: SessionData) {
        self.store.write().await.insert(session_id.to_string(), data);
    }

    /// Remove a session from the store.
    pub async fn destroy_session(&self, session_id: &str) {
        self.store.write().await.remove(session_id);
    }

    /// Return a Set-Cookie header value that clears the session cookie.
    pub fn clear_cookie(&self) -> String {
        let mut cookie = format!(
            "{}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0",
            self.cookie_name
        );
        if self.secure {
            cookie.push_str("; Secure");
        }
        if !self.cookie_domain.is_empty() {
            cookie.push_str(&format!("; Domain={}", self.cookie_domain));
        }
        cookie
    }

    /// Check if the access token in the session is expired (or about to expire).
    pub fn is_token_expired(session: &SessionData) -> bool {
        let now = chrono::Utc::now().timestamp();
        // Consider expired if within 60 seconds of expiry
        session.expires_at <= now + 60
    }

    /// Start a background task to periodically clean up expired sessions.
    pub fn start_cleanup_task(&self, interval_secs: u64) {
        let store = Arc::clone(&self.store);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
                let now = chrono::Utc::now().timestamp();
                let mut sessions = store.write().await;
                let before = sessions.len();
                sessions.retain(|_, s| s.expires_at > now);
                let removed = before - sessions.len();
                if removed > 0 {
                    tracing::debug!("Session cleanup: removed {} expired sessions", removed);
                }
            }
        });
    }

    // ── Private helpers ──────────────────────────────────────────────

    /// Encrypt a session ID into a cookie value.
    fn encrypt_cookie(&self, session_id: &str) -> Result<String, String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, session_id.as_bytes())
            .map_err(|e| format!("Session encrypt error: {}", e))?;

        let mut payload = Vec::with_capacity(12 + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);
        let encoded = URL_SAFE_NO_PAD.encode(&payload);

        tracing::debug!("Session cookie size: {} bytes", encoded.len() + self.cookie_name.len() + 1);

        let mut cookie = format!(
            "{}={}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}",
            self.cookie_name, encoded, self.max_age_seconds
        );
        if self.secure {
            cookie.push_str("; Secure");
        }
        if !self.cookie_domain.is_empty() {
            cookie.push_str(&format!("; Domain={}", self.cookie_domain));
        }

        Ok(cookie)
    }
}

/// Refresh an access token using the refresh token via CoreAuth.
pub async fn refresh_access_token(
    http_client: &reqwest::Client,
    coreauth_url: &str,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenResponse, String> {
    let url = format!("{}/oauth/token", coreauth_url);

    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
    ];

    let resp = http_client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed ({}): {}", status, body));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| format!("Token refresh parse error: {}", e))
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    #[allow(dead_code)]
    pub token_type: String,
    pub expires_in: i64,
}
