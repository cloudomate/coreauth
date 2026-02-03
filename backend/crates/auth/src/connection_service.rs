use crate::error::{AuthError, Result};
use ciam_database::{ConnectionRepository, Database};
use ciam_models::{Connection, ConnectionScope, CreateConnection, UpdateConnection};
use uuid::Uuid;

pub struct ConnectionService {
    db: Database,
    connection_repo: ConnectionRepository,
}

impl ConnectionService {
    pub fn new(db: Database) -> Self {
        let pool = db.pool().clone();

        Self {
            db,
            connection_repo: ConnectionRepository::new(pool),
        }
    }

    /// Create a new connection
    pub async fn create(&self, request: CreateConnection) -> Result<Connection> {
        // Validate connection configuration based on type
        self.validate_connection_config(&request.connection_type, &request.config)?;

        // Ensure scope matches organization_id
        match request.scope {
            ConnectionScope::Platform => {
                if request.organization_id.is_some() {
                    return Err(AuthError::InvalidInput(
                        "Platform-level connections cannot have an organization_id".to_string(),
                    ));
                }
            }
            ConnectionScope::Organization => {
                if request.organization_id.is_none() {
                    return Err(AuthError::InvalidInput(
                        "Organization-level connections must have an organization_id".to_string(),
                    ));
                }
            }
        }

        self.connection_repo
            .create(request)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Get connection by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Connection> {
        self.connection_repo
            .get_by_id(id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or_else(|| AuthError::NotFound("Connection not found".to_string()))
    }

    /// Get all platform-level connections
    pub async fn get_platform_connections(&self) -> Result<Vec<Connection>> {
        self.connection_repo
            .get_platform_connections()
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Get all connections for an organization
    pub async fn get_organization_connections(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Connection>> {
        self.connection_repo
            .get_by_organization(organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Get primary SSO connection for an organization
    pub async fn get_organization_sso(&self, organization_id: Uuid) -> Result<Option<Connection>> {
        self.connection_repo
            .get_org_sso_connection(organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Get database connection (platform-level)
    pub async fn get_database_connection(&self) -> Result<Option<Connection>> {
        self.connection_repo
            .get_database_connection()
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Resolve connections for user login
    /// Priority: Org-level SSO > Platform SSO > Database
    pub async fn resolve_for_login(
        &self,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<Connection>> {
        self.connection_repo
            .resolve_for_login(organization_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Get available authentication methods for a login context
    /// Returns a list of connection types that the user can use
    pub async fn get_available_auth_methods(
        &self,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<AuthMethod>> {
        let connections = self.resolve_for_login(organization_id).await?;

        let methods = connections
            .iter()
            .map(|conn| AuthMethod {
                connection_id: conn.id,
                name: conn.name.clone(),
                method_type: conn.connection_type.clone(),
                scope: conn.scope.clone(),
            })
            .collect();

        Ok(methods)
    }

    /// Update connection
    pub async fn update(&self, id: Uuid, updates: UpdateConnection) -> Result<Connection> {
        // Validate config if provided
        if let Some(config) = &updates.config {
            // Get existing connection to know the type
            let existing = self.get_by_id(id).await?;
            self.validate_connection_config(&existing.connection_type, config)?;
        }

        self.connection_repo
            .update(id, updates)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Delete connection
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let deleted = self
            .connection_repo
            .delete(id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        if !deleted {
            return Err(AuthError::NotFound("Connection not found".to_string()));
        }

        Ok(())
    }

    /// List all connections (admin only)
    pub async fn list_all(&self) -> Result<Vec<Connection>> {
        self.connection_repo
            .list_all()
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Count connections by scope
    pub async fn count_by_scope(&self, scope: ConnectionScope) -> Result<i64> {
        self.connection_repo
            .count_by_scope(scope)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    /// Validate connection configuration based on type
    fn validate_connection_config(
        &self,
        connection_type: &str,
        config: &serde_json::Value,
    ) -> Result<()> {
        match connection_type {
            "database" => {
                // Database connections have minimal config
                Ok(())
            }
            "oidc" => {
                // Validate OIDC config has required fields
                let issuer = config.get("issuer").and_then(|v| v.as_str());
                let client_id = config.get("client_id").and_then(|v| v.as_str());
                let client_secret = config.get("client_secret").and_then(|v| v.as_str());

                if issuer.is_none() || client_id.is_none() || client_secret.is_none() {
                    return Err(AuthError::InvalidInput(
                        "OIDC connection requires issuer, client_id, and client_secret".to_string(),
                    ));
                }

                Ok(())
            }
            "saml" => {
                // Validate SAML config has required fields
                let entity_id = config.get("entity_id").and_then(|v| v.as_str());
                let sso_url = config.get("sso_url").and_then(|v| v.as_str());
                let x509_cert = config.get("x509_cert").and_then(|v| v.as_str());

                if entity_id.is_none() || sso_url.is_none() || x509_cert.is_none() {
                    return Err(AuthError::InvalidInput(
                        "SAML connection requires entity_id, sso_url, and x509_cert".to_string(),
                    ));
                }

                Ok(())
            }
            "oauth2" => {
                // Validate OAuth2 config has required fields
                let auth_url = config.get("authorization_url").and_then(|v| v.as_str());
                let token_url = config.get("token_url").and_then(|v| v.as_str());
                let client_id = config.get("client_id").and_then(|v| v.as_str());
                let client_secret = config.get("client_secret").and_then(|v| v.as_str());

                if auth_url.is_none()
                    || token_url.is_none()
                    || client_id.is_none()
                    || client_secret.is_none()
                {
                    return Err(AuthError::InvalidInput(
                        "OAuth2 connection requires authorization_url, token_url, client_id, and client_secret"
                            .to_string(),
                    ));
                }

                Ok(())
            }
            _ => Err(AuthError::InvalidInput(format!(
                "Unsupported connection type: {}. Must be one of: database, oidc, saml, oauth2",
                connection_type
            ))),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthMethod {
    pub connection_id: Uuid,
    pub name: String,
    pub method_type: String, // 'database', 'oidc', 'saml', 'oauth2'
    pub scope: ConnectionScope,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_oidc_config() {
        let service = ConnectionService {
            db: Database::new_test(), // Assume this exists
            connection_repo: ConnectionRepository::new_test(),
        };

        // Valid OIDC config
        let valid_config = serde_json::json!({
            "issuer": "https://accounts.google.com",
            "client_id": "client123",
            "client_secret": "secret456"
        });

        assert!(service
            .validate_connection_config("oidc", &valid_config)
            .is_ok());

        // Invalid OIDC config (missing client_secret)
        let invalid_config = serde_json::json!({
            "issuer": "https://accounts.google.com",
            "client_id": "client123"
        });

        assert!(service
            .validate_connection_config("oidc", &invalid_config)
            .is_err());
    }

    #[test]
    fn test_validate_saml_config() {
        let service = ConnectionService {
            db: Database::new_test(),
            connection_repo: ConnectionRepository::new_test(),
        };

        // Valid SAML config
        let valid_config = serde_json::json!({
            "entity_id": "https://saml.example.com/entity",
            "sso_url": "https://saml.example.com/sso",
            "x509_cert": "-----BEGIN CERTIFICATE-----\nMIIC..."
        });

        assert!(service
            .validate_connection_config("saml", &valid_config)
            .is_ok());

        // Invalid SAML config (missing x509_cert)
        let invalid_config = serde_json::json!({
            "entity_id": "https://saml.example.com/entity",
            "sso_url": "https://saml.example.com/sso"
        });

        assert!(service
            .validate_connection_config("saml", &invalid_config)
            .is_err());
    }
}
