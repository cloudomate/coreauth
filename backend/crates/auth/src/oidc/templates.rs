use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProviderTemplate {
    pub provider_type: String,
    pub display_name: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: String,
    pub scopes: Vec<String>,
    pub groups_claim: String,
    pub issuer_pattern: String, // Pattern with {domain} placeholder
    pub instructions: String,
}

/// Get provider template by type
pub fn get_provider_template(provider_type: &str) -> Option<OidcProviderTemplate> {
    match provider_type {
        "auth0" => Some(auth0_template()),
        "google" => Some(google_workspace_template()),
        "azuread" | "entra" => Some(azure_ad_template()),
        _ => None,
    }
}

/// List all available provider templates
pub fn list_provider_templates() -> Vec<OidcProviderTemplate> {
    vec![
        auth0_template(),
        google_workspace_template(),
        azure_ad_template(),
    ]
}

fn auth0_template() -> OidcProviderTemplate {
    OidcProviderTemplate {
        provider_type: "auth0".to_string(),
        display_name: "Auth0".to_string(),
        authorization_endpoint: "https://{domain}/authorize".to_string(),
        token_endpoint: "https://{domain}/oauth/token".to_string(),
        userinfo_endpoint: Some("https://{domain}/userinfo".to_string()),
        jwks_uri: "https://{domain}/.well-known/jwks.json".to_string(),
        scopes: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        groups_claim: "https://schemas.auth0.com/groups".to_string(),
        issuer_pattern: "https://{domain}/".to_string(),
        instructions: "Replace {domain} with your Auth0 tenant domain (e.g., your-tenant.us.auth0.com). In Auth0, create an application and get your Client ID and Client Secret. Add groups to user metadata and create an Action to include groups in the token.".to_string(),
    }
}

fn google_workspace_template() -> OidcProviderTemplate {
    OidcProviderTemplate {
        provider_type: "google".to_string(),
        display_name: "Google Workspace".to_string(),
        authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        token_endpoint: "https://oauth2.googleapis.com/token".to_string(),
        userinfo_endpoint: Some("https://openidconnect.googleapis.com/v1/userinfo".to_string()),
        jwks_uri: "https://www.googleapis.com/oauth2/v3/certs".to_string(),
        scopes: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        groups_claim: "groups".to_string(),
        issuer_pattern: "https://accounts.google.com".to_string(),
        instructions: "Create OAuth 2.0 credentials in Google Cloud Console. Add authorized redirect URIs. For groups, you need to enable the Admin SDK and request the 'https://www.googleapis.com/auth/admin.directory.group.readonly' scope, or use a custom claim.".to_string(),
    }
}

fn azure_ad_template() -> OidcProviderTemplate {
    OidcProviderTemplate {
        provider_type: "azuread".to_string(),
        display_name: "Microsoft Entra ID (Azure AD)".to_string(),
        authorization_endpoint: "https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/authorize".to_string(),
        token_endpoint: "https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token".to_string(),
        userinfo_endpoint: None,
        jwks_uri: "https://login.microsoftonline.com/{tenant_id}/discovery/v2.0/keys".to_string(),
        scopes: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        groups_claim: "groups".to_string(),
        issuer_pattern: "https://login.microsoftonline.com/{tenant_id}/v2.0".to_string(),
        instructions: "Replace {tenant_id} with your Azure AD tenant ID. In Azure Portal, register an application and get Application (client) ID and create a client secret. Under Token Configuration, add 'groups' as an optional claim to include groups in ID tokens.".to_string(),
    }
}

/// Replace placeholders in URLs
pub fn apply_template_values(template: &str, values: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in values {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_provider_template() {
        let auth0 = get_provider_template("auth0").unwrap();
        assert_eq!(auth0.provider_type, "auth0");
        assert!(auth0.authorization_endpoint.contains("{domain}"));

        let google = get_provider_template("google").unwrap();
        assert_eq!(google.provider_type, "google");

        let azure = get_provider_template("azuread").unwrap();
        assert_eq!(azure.provider_type, "azuread");
    }

    #[test]
    fn test_apply_template_values() {
        let mut values = HashMap::new();
        values.insert("domain".to_string(), "my-tenant.auth0.com".to_string());

        let result = apply_template_values("https://{domain}/authorize", &values);
        assert_eq!(result, "https://my-tenant.auth0.com/authorize");
    }

    #[test]
    fn test_list_provider_templates() {
        let templates = list_provider_templates();
        assert_eq!(templates.len(), 3);
    }
}
