use crate::error::{DatabaseError, Result};
use ciam_models::{Action, ActionExecution, ActionTrigger, CreateAction, ExecutionStatus, UpdateAction};
use sqlx::PgPool;
use uuid::Uuid;

pub struct ActionRepository {
    pool: PgPool,
}

impl ActionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new action
    pub async fn create(&self, request: &CreateAction) -> Result<Action> {
        let runtime = request.runtime.clone().unwrap_or_else(|| "nodejs18".to_string());
        let timeout_seconds = request.timeout_seconds.unwrap_or(10);
        let secrets = request.secrets.clone().unwrap_or(serde_json::json!({}));
        let execution_order = request.execution_order.unwrap_or(0);

        let action = sqlx::query_as::<_, Action>(
            r#"
            INSERT INTO actions (
                organization_id, name, description, trigger_type, code,
                runtime, timeout_seconds, secrets, execution_order, is_enabled
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(&request.organization_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.trigger_type)
        .bind(&request.code)
        .bind(&runtime)
        .bind(timeout_seconds)
        .bind(sqlx::types::Json(&secrets))
        .bind(execution_order)
        .bind(true) // is_enabled by default
        .fetch_one(&self.pool)
        .await?;

        Ok(action)
    }

    /// Get action by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Action> {
        let action = sqlx::query_as::<_, Action>("SELECT * FROM actions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| DatabaseError::not_found("Action", &id.to_string()))?;

        Ok(action)
    }

    /// List all actions for an organization
    pub async fn list_by_organization(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Action>> {
        let actions = sqlx::query_as::<_, Action>(
            r#"
            SELECT * FROM actions
            WHERE organization_id = $1
            ORDER BY trigger_type, execution_order, created_at
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(org_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(actions)
    }

    /// List actions by trigger type (ordered by execution_order)
    pub async fn list_by_trigger(
        &self,
        org_id: Uuid,
        trigger: ActionTrigger,
    ) -> Result<Vec<Action>> {
        let actions = sqlx::query_as::<_, Action>(
            r#"
            SELECT * FROM actions
            WHERE organization_id = $1 AND trigger_type = $2 AND is_enabled = true
            ORDER BY execution_order, created_at
            "#,
        )
        .bind(org_id)
        .bind(&trigger)
        .fetch_all(&self.pool)
        .await?;

        Ok(actions)
    }

    /// Update action
    pub async fn update(&self, id: Uuid, request: &UpdateAction) -> Result<Action> {
        let current = self.get_by_id(id).await?;

        let name = request.name.as_ref().unwrap_or(&current.name);
        let description = request.description.as_ref().or(current.description.as_ref());
        let code = request.code.as_ref().unwrap_or(&current.code);
        let runtime = request.runtime.as_ref().unwrap_or(&current.runtime);
        let timeout_seconds = request.timeout_seconds.unwrap_or(current.timeout_seconds);
        let secrets = request.secrets.as_ref().unwrap_or(&current.secrets);
        let execution_order = request.execution_order.unwrap_or(current.execution_order);
        let is_enabled = request.is_enabled.unwrap_or(current.is_enabled);

        let action = sqlx::query_as::<_, Action>(
            r#"
            UPDATE actions
            SET name = $1, description = $2, code = $3, runtime = $4,
                timeout_seconds = $5, secrets = $6, execution_order = $7,
                is_enabled = $8, updated_at = NOW()
            WHERE id = $9
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(code)
        .bind(runtime)
        .bind(timeout_seconds)
        .bind(sqlx::types::Json(secrets))
        .bind(execution_order)
        .bind(is_enabled)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(action)
    }

    /// Delete action
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM actions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Record action execution in partitioned table
    pub async fn record_execution(
        &self,
        action_id: Uuid,
        org_id: Uuid,
        trigger: &str,
        user_id: Option<Uuid>,
        status: ExecutionStatus,
        execution_time_ms: i32,
        input_data: Option<serde_json::Value>,
        output_data: Option<serde_json::Value>,
        error_message: Option<String>,
    ) -> Result<ActionExecution> {
        let execution = sqlx::query_as::<_, ActionExecution>(
            r#"
            INSERT INTO action_executions (
                action_id, organization_id, trigger_type, user_id,
                status, execution_time_ms, input_data, output_data, error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(action_id)
        .bind(org_id)
        .bind(trigger)
        .bind(user_id)
        .bind(&status)
        .bind(execution_time_ms)
        .bind(sqlx::types::Json(&input_data))
        .bind(sqlx::types::Json(&output_data))
        .bind(&error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(execution)
    }

    /// Update action statistics after execution
    pub async fn update_stats(&self, action_id: Uuid, success: bool) -> Result<()> {
        if success {
            sqlx::query(
                r#"
                UPDATE actions
                SET last_executed_at = NOW(),
                    total_executions = total_executions + 1,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(action_id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                UPDATE actions
                SET last_executed_at = NOW(),
                    total_executions = total_executions + 1,
                    total_failures = total_failures + 1,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(action_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get execution history for an action (paginated)
    pub async fn get_executions(
        &self,
        action_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ActionExecution>> {
        let executions = sqlx::query_as::<_, ActionExecution>(
            r#"
            SELECT * FROM action_executions
            WHERE action_id = $1
            ORDER BY executed_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(action_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(executions)
    }

    /// Get recent executions for an organization (all actions)
    pub async fn get_organization_executions(
        &self,
        org_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ActionExecution>> {
        let executions = sqlx::query_as::<_, ActionExecution>(
            r#"
            SELECT * FROM action_executions
            WHERE organization_id = $1
            ORDER BY executed_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(org_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(executions)
    }

    /// Count actions by organization
    pub async fn count_by_organization(&self, org_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM actions WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }
}
