use crate::action_executor::ActionExecutor;
use crate::error::{AuthError, Result};
use ciam_database::repositories::action::ActionRepository;
use ciam_database::Database;
use ciam_models::{
    Action, ActionContext, ActionExecution, ActionResult, ActionTrigger, CreateAction,
    ExecutionStatus, UpdateAction,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub struct ActionService {
    db: Database,
    action_repo: ActionRepository,
    executor: Arc<ActionExecutor>,
}

impl ActionService {
    pub fn new(db: Database) -> Self {
        let pool = db.pool().clone();

        Self {
            db,
            action_repo: ActionRepository::new(pool),
            executor: Arc::new(ActionExecutor::new()),
        }
    }

    /// Create a new action
    pub async fn create(&self, request: CreateAction, actor_user_id: Uuid) -> Result<Action> {
        // Validate request
        request
            .validate()
            .map_err(|e| AuthError::InvalidInput(e.to_string()))?;

        // TODO: Validate JavaScript syntax before saving

        // Create action
        let action = self
            .action_repo
            .create(&request)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        Ok(action)
    }

    /// Get action by ID
    pub async fn get(&self, id: Uuid, actor_user_id: Uuid) -> Result<Action> {
        let action = self
            .action_repo
            .get_by_id(id)
            .await
            .map_err(|e| match e {
                ciam_database::error::DatabaseError::NotFound(_) => {
                    AuthError::NotFound("Action not found".to_string())
                }
                _ => AuthError::Internal(e.to_string()),
            })?;

        // TODO: Check if actor has access to this action's organization

        Ok(action)
    }

    /// Update action
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdateAction,
        actor_user_id: Uuid,
    ) -> Result<Action> {
        // Validate request
        request
            .validate()
            .map_err(|e| AuthError::InvalidInput(e.to_string()))?;

        // Get existing action
        let existing = self.action_repo.get_by_id(id).await.map_err(|e| match e {
            ciam_database::error::DatabaseError::NotFound(_) => {
                AuthError::NotFound("Action not found".to_string())
            }
            _ => AuthError::Internal(e.to_string()),
        })?;

        // TODO: Check if actor has access to this action's organization
        // TODO: Validate JavaScript syntax if code is being updated

        // Update action
        let action = self
            .action_repo
            .update(id, &request)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        Ok(action)
    }

    /// Delete action
    pub async fn delete(&self, id: Uuid, actor_user_id: Uuid) -> Result<()> {
        // Get existing action for audit
        let existing = self.action_repo.get_by_id(id).await.map_err(|e| match e {
            ciam_database::error::DatabaseError::NotFound(_) => {
                AuthError::NotFound("Action not found".to_string())
            }
            _ => AuthError::Internal(e.to_string()),
        })?;

        // TODO: Check if actor has access to this action's organization

        // Delete action
        self.action_repo
            .delete(id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        Ok(())
    }

    /// List all actions for an organization
    pub async fn list_by_organization(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Action>> {
        // TODO: Check if actor has access to this organization

        let actions = self
            .action_repo
            .list_by_organization(org_id, limit, offset)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(actions)
    }

    /// Test action with sample data (doesn't record execution)
    pub async fn test_action(
        &self,
        id: Uuid,
        context: ActionContext,
        actor_user_id: Uuid,
    ) -> Result<ActionResult> {
        let action = self.action_repo.get_by_id(id).await.map_err(|e| match e {
            ciam_database::error::DatabaseError::NotFound(_) => {
                AuthError::NotFound("Action not found".to_string())
            }
            _ => AuthError::Internal(e.to_string()),
        })?;

        // TODO: Check if actor has access to this action's organization

        // Execute action using the executor
        let result = self.executor.execute(&action, context).await?;

        Ok(result)
    }

    /// Execute all actions for a specific trigger
    /// This is called during auth flows (login, registration, etc.)
    pub async fn execute_trigger(
        &self,
        org_id: Uuid,
        trigger: ActionTrigger,
        context: ActionContext,
    ) -> Result<Vec<ActionResult>> {
        let trigger_str = trigger.to_string();

        // Get all enabled actions for this trigger, ordered by execution_order
        let actions = self
            .action_repo
            .list_by_trigger(org_id, trigger)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        let mut results = Vec::new();

        for action in actions {
            // Execute action using the executor
            let start_time = std::time::Instant::now();
            let result = self.executor.execute(&action, context.clone()).await?;
            let execution_time_ms = start_time.elapsed().as_millis() as i32;

            // Record execution (even placeholder for now)
            let status = if result.success {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Failure
            };

            let user_id = context.user.as_ref().and_then(|u| {
                u.get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
            });

            self.action_repo
                .record_execution(
                    action.id,
                    org_id,
                    &trigger_str,
                    user_id,
                    status.clone(),
                    execution_time_ms,
                    Some(serde_json::to_value(&context).unwrap_or_default()),
                    Some(result.data.clone()),
                    result.error.clone(),
                )
                .await
                .ok();

            // Update action stats
            self.action_repo
                .update_stats(action.id, result.success)
                .await
                .ok();

            results.push(result);
        }

        Ok(results)
    }

    /// Get execution history for an action
    pub async fn get_executions(
        &self,
        action_id: Uuid,
        limit: i64,
        offset: i64,
        actor_user_id: Uuid,
    ) -> Result<Vec<ActionExecution>> {
        // Get action to check permissions
        let action = self.action_repo.get_by_id(action_id).await.map_err(|e| {
            match e {
                ciam_database::error::DatabaseError::NotFound(_) => {
                    AuthError::NotFound("Action not found".to_string())
                }
                _ => AuthError::Internal(e.to_string()),
            }
        })?;

        // TODO: Check if actor has access to this action's organization

        let executions = self
            .action_repo
            .get_executions(action_id, limit, offset)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(executions)
    }

    /// Get recent executions for an organization (all actions)
    pub async fn get_organization_executions(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ActionExecution>> {
        // TODO: Check if actor has access to this organization

        let executions = self
            .action_repo
            .get_organization_executions(org_id, limit, offset)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(executions)
    }

    /// Count actions by organization
    pub async fn count_by_organization(&self, org_id: Uuid) -> Result<i64> {
        let count = self
            .action_repo
            .count_by_organization(org_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(count)
    }
}
