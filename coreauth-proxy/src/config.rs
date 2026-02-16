use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ProxyConfig {
    pub server: ServerConfig,
    pub coreauth: CoreAuthConfig,
    pub session: SessionConfig,
    #[serde(default)]
    pub fga: FgaConfig,
    #[serde(default)]
    pub routes: Vec<RouteRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_listen")]
    pub listen: String,
    pub upstream: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoreAuthConfig {
    pub url: String,
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfig {
    pub secret: String,
    #[serde(default = "default_cookie_name")]
    pub cookie_name: String,
    #[serde(default)]
    pub cookie_domain: String,
    #[serde(default = "default_max_age")]
    pub max_age_seconds: u64,
    #[serde(default)]
    pub secure: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FgaConfig {
    #[serde(default)]
    pub store_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouteRule {
    #[serde(rename = "match")]
    pub match_rule: MatchRule,
    #[serde(default = "default_auth_mode")]
    pub auth: AuthMode,
    #[serde(default)]
    pub on_unauthenticated: UnauthAction,
    #[serde(default)]
    pub target: RouteTarget,
    pub fga: Option<FgaRule>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RouteTarget {
    #[default]
    Upstream,
    Coreauth,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchRule {
    pub path: String,
    #[serde(default)]
    pub methods: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    None,
    Optional,
    Required,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UnauthAction {
    #[default]
    RedirectLogin,
    Status401,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FgaRule {
    pub relation: String,
    pub object_type: String,
    /// Where to extract the object ID from. Format: "path:<param>" or "query:<param>" or "header:<name>"
    pub object_id: String,
}

fn default_listen() -> String {
    "0.0.0.0:4000".to_string()
}

fn default_cookie_name() -> String {
    "coreauth_session".to_string()
}

fn default_max_age() -> u64 {
    86400
}

fn default_auth_mode() -> AuthMode {
    AuthMode::Required
}

impl ProxyConfig {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ProxyConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
