use crate::error::{AuthError, Result};
use ciam_database::repositories::application::ApplicationRepository;
use ciam_database::Database;
use ciam_models::{Application, ApplicationWithSecret, CreateApplication, UpdateApplication};
use uuid::Uuid;
use validator::Validate;

pub struct ApplicationService {
    db: Database,
    app_repo: ApplicationRepository,
}

impl ApplicationService {
    pub fn new(db: Database) -> Self {
        let pool = db.pool().clone();

        Self {
            db,
            app_repo: ApplicationRepository::new(pool),
        }
    }

    /// Create a new application
    pub async fn create(
        &self,
        request: CreateApplication,
        actor_user_id: Uuid,
    ) -> Result<ApplicationWithSecret> {
        // Validate request
        request
            .validate()
            .map_err(|e| AuthError::InvalidInput(e.to_string()))?;

        // Create application
        let app_with_secret = self
            .app_repo
            .create(&request)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        // TODO: Add audit logging

        Ok(app_with_secret)
    }

    /// Get application by ID
    pub async fn get(&self, id: Uuid, actor_user_id: Uuid) -> Result<Application> {
        let app = self
            .app_repo
            .get_by_id(id)
            .await
            .map_err(|e| match e {
                ciam_database::error::DatabaseError::NotFound(_) => {
                    AuthError::NotFound("Application not found".to_string())
                }
                _ => AuthError::Internal(e.to_string()),
            })?;

        // TODO: Check if actor has access to this application's organization

        Ok(app)
    }

    /// Update application
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdateApplication,
        actor_user_id: Uuid,
    ) -> Result<Application> {
        // Validate request
        request
            .validate()
            .map_err(|e| AuthError::InvalidInput(e.to_string()))?;

        // Get existing app
        let existing = self.app_repo.get_by_id(id).await.map_err(|e| match e {
            ciam_database::error::DatabaseError::NotFound(_) => {
                AuthError::NotFound("Application not found".to_string())
            }
            _ => AuthError::Internal(e.to_string()),
        })?;

        // TODO: Check if actor has access to this application's organization

        // Update application
        let app = self
            .app_repo
            .update(id, &request)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        Ok(app)
    }

    /// Delete application
    pub async fn delete(&self, id: Uuid, actor_user_id: Uuid) -> Result<()> {
        // Get existing app for audit
        let existing = self.app_repo.get_by_id(id).await.map_err(|e| match e {
            ciam_database::error::DatabaseError::NotFound(_) => {
                AuthError::NotFound("Application not found".to_string())
            }
            _ => AuthError::Internal(e.to_string()),
        })?;

        // TODO: Check if actor has access to this application's organization

        // Delete application
        self.app_repo
            .delete(id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Rotate application client secret
    pub async fn rotate_secret(
        &self,
        id: Uuid,
        actor_user_id: Uuid,
    ) -> Result<ApplicationWithSecret> {
        // Get existing app
        let existing = self.app_repo.get_by_id(id).await.map_err(|e| match e {
            ciam_database::error::DatabaseError::NotFound(_) => {
                AuthError::NotFound("Application not found".to_string())
            }
            _ => AuthError::Internal(e.to_string()),
        })?;

        // TODO: Check if actor has access to this application's organization

        // Rotate secret
        let app_with_secret = self
            .app_repo
            .rotate_secret(id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        Ok(app_with_secret)
    }

    /// List applications for an organization
    pub async fn list_by_organization(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Application>> {
        // TODO: Check if actor has access to this organization

        let apps = self
            .app_repo
            .list_by_organization(org_id, limit, offset)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(apps)
    }

    /// List global/platform-level applications
    pub async fn list_global(&self, limit: i64, offset: i64) -> Result<Vec<Application>> {
        // TODO: Check if actor is platform admin

        let apps = self
            .app_repo
            .list_global(limit, offset)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(apps)
    }

    /// Authenticate application (for OAuth flows)
    pub async fn authenticate(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<Application> {
        let app = self
            .app_repo
            .verify_credentials(client_id, client_secret)
            .await
            .map_err(|e| match e {
                ciam_database::error::DatabaseError::Unauthorized(_) => {
                    AuthError::InvalidCredentials("Invalid client credentials".into())
                }
                ciam_database::error::DatabaseError::NotFound(_) => {
                    AuthError::InvalidCredentials("Application not found".into())
                }
                _ => AuthError::Internal(e.to_string()),
            })?;

        // Check if application is enabled
        if !app.is_enabled {
            return Err(AuthError::Forbidden(
                "Application is disabled".to_string(),
            ));
        }

        Ok(app)
    }

    /// Count applications by organization
    pub async fn count_by_organization(&self, org_id: Uuid) -> Result<i64> {
        let count = self
            .app_repo
            .count_by_organization(org_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(count)
    }
}
