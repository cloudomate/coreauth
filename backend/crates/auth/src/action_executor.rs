use crate::error::{AuthError, Result};
use ciam_models::{Action, ActionContext, ActionResult};
use deno_core::{JsRuntime, RuntimeOptions};
use std::time::{Duration, Instant};

pub struct ActionExecutor;

impl ActionExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a JavaScript action with timeout enforcement
    pub async fn execute(
        &self,
        action: &Action,
        context: ActionContext,
    ) -> Result<ActionResult> {
        let timeout_duration = Duration::from_secs(action.timeout_seconds as u64);
        let start_time = Instant::now();

        // Execute in blocking task to avoid blocking async runtime
        let action_code = action.code.clone();
        let action_secrets = action.secrets.clone();
        let context_clone = context.clone();

        // Spawn blocking task with timeout
        let execution_handle = tokio::task::spawn_blocking(move || {
            Self::execute_sync(&action_code, &action_secrets, &context_clone)
        });

        // Wait for execution with timeout
        let result = match tokio::time::timeout(timeout_duration, execution_handle).await {
            Ok(Ok(exec_result)) => exec_result,
            Ok(Err(e)) => {
                return Ok(ActionResult {
                    success: false,
                    data: serde_json::json!({
                        "execution_time_ms": start_time.elapsed().as_millis() as i32
                    }),
                    error: Some(format!("Failed to execute action: {}", e)),
                });
            }
            Err(_) => {
                return Ok(ActionResult {
                    success: false,
                    data: serde_json::json!({
                        "execution_time_ms": start_time.elapsed().as_millis() as i32
                    }),
                    error: Some(format!(
                        "Action execution timed out after {}s",
                        timeout_duration.as_secs()
                    )),
                });
            }
        };

        let execution_time_ms = start_time.elapsed().as_millis() as i32;

        match result {
            Ok(data) => Ok(ActionResult {
                success: true,
                data,
                error: None,
            }),
            Err(error_msg) => Ok(ActionResult {
                success: false,
                data: serde_json::json!({ "execution_time_ms": execution_time_ms }),
                error: Some(error_msg),
            }),
        }
    }

    /// Execute JavaScript code synchronously using Deno V8 runtime
    fn execute_sync(
        code: &str,
        secrets: &serde_json::Value,
        context: &ActionContext,
    ) -> std::result::Result<serde_json::Value, String> {
        // Create a new V8 isolate with Deno runtime
        let mut runtime = JsRuntime::new(RuntimeOptions {
            ..Default::default()
        });

        // Serialize context and secrets to JSON strings
        let context_json = serde_json::to_string(context)
            .map_err(|e| format!("Failed to serialize context: {}", e))?;
        let secrets_json = serde_json::to_string(secrets)
            .map_err(|e| format!("Failed to serialize secrets: {}", e))?;

        // Build the complete JavaScript code with context injection
        let full_code = format!(
            r#"
            // Inject context and secrets as global variables
            globalThis.context = {};
            globalThis.secrets = {};

            // Execute user code in an IIFE to capture return value
            (function() {{
                {}
            }})();
            "#,
            context_json, secrets_json, code
        );

        // Execute the code and get the result
        let result = runtime
            .execute_script("<action>", full_code.into())
            .map_err(|e| format!("JavaScript execution error: {}", e))?;

        // Convert V8 value to serde_json::Value
        let scope = &mut runtime.handle_scope();
        let local = deno_core::v8::Local::new(scope, result);

        let json_value = serde_v8::from_v8::<serde_json::Value>(scope, local)
            .map_err(|e| format!("Failed to convert result to JSON: {}", e))?;

        Ok(json_value)
    }
}

impl Default for ActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciam_models::ActionContext;

    #[tokio::test]
    async fn test_simple_action_execution() {
        let executor = ActionExecutor::new();
        let action = Action {
            id: uuid::Uuid::new_v4(),
            organization_id: uuid::Uuid::new_v4(),
            name: "Test Action".to_string(),
            description: None,
            trigger_type: ciam_models::ActionTrigger::PostLogin,
            code: r#"
                return {
                    success: true,
                    message: "Hello from action",
                    user_email: context.user ? context.user.email : null
                };
            "#
            .to_string(),
            runtime: "nodejs18".to_string(),
            timeout_seconds: 5,
            secrets: serde_json::json!({}),
            execution_order: 0,
            is_enabled: true,
            last_executed_at: None,
            total_executions: 0,
            total_failures: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let context = ActionContext {
            user: Some(serde_json::json!({
                "id": "123",
                "email": "test@example.com"
            })),
            organization: None,
            event: "login".to_string(),
            metadata: serde_json::json!({}),
        };

        let result = executor.execute(&action, context).await.unwrap();

        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_action_with_error() {
        let executor = ActionExecutor::new();
        let action = Action {
            id: uuid::Uuid::new_v4(),
            organization_id: uuid::Uuid::new_v4(),
            name: "Failing Action".to_string(),
            description: None,
            trigger_type: ciam_models::ActionTrigger::PreLogin,
            code: r#"
                throw new Error("This action failed intentionally");
            "#
            .to_string(),
            runtime: "nodejs18".to_string(),
            timeout_seconds: 5,
            secrets: serde_json::json!({}),
            execution_order: 0,
            is_enabled: true,
            last_executed_at: None,
            total_executions: 0,
            total_failures: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let context = ActionContext {
            user: None,
            organization: None,
            event: "login".to_string(),
            metadata: serde_json::json!({}),
        };

        let result = executor.execute(&action, context).await.unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_action_timeout() {
        let executor = ActionExecutor::new();
        let action = Action {
            id: uuid::Uuid::new_v4(),
            organization_id: uuid::Uuid::new_v4(),
            name: "Timeout Action".to_string(),
            description: None,
            trigger_type: ciam_models::ActionTrigger::PreLogin,
            code: r#"
                // Infinite loop to trigger timeout
                while(true) {}
            "#
            .to_string(),
            runtime: "nodejs18".to_string(),
            timeout_seconds: 1, // 1 second timeout
            secrets: serde_json::json!({}),
            execution_order: 0,
            is_enabled: true,
            last_executed_at: None,
            total_executions: 0,
            total_failures: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let context = ActionContext {
            user: None,
            organization: None,
            event: "login".to_string(),
            metadata: serde_json::json!({}),
        };

        let result = executor.execute(&action, context).await.unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("timed out"));
    }

    #[tokio::test]
    async fn test_context_and_secrets_injection() {
        let executor = ActionExecutor::new();
        let action = Action {
            id: uuid::Uuid::new_v4(),
            organization_id: uuid::Uuid::new_v4(),
            name: "Context Test".to_string(),
            description: None,
            trigger_type: ciam_models::ActionTrigger::PostLogin,
            code: r#"
                return {
                    user_id: context.user.id,
                    event: context.event,
                    api_key: secrets.api_key
                };
            "#
            .to_string(),
            runtime: "nodejs18".to_string(),
            timeout_seconds: 5,
            secrets: serde_json::json!({
                "api_key": "test_secret_key_123"
            }),
            execution_order: 0,
            is_enabled: true,
            last_executed_at: None,
            total_executions: 0,
            total_failures: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let context = ActionContext {
            user: Some(serde_json::json!({
                "id": "user-123",
                "email": "test@example.com"
            })),
            organization: None,
            event: "login".to_string(),
            metadata: serde_json::json!({}),
        };

        let result = executor.execute(&action, context).await.unwrap();

        assert!(result.success);
        assert_eq!(result.data["user_id"], "user-123");
        assert_eq!(result.data["event"], "login");
        assert_eq!(result.data["api_key"], "test_secret_key_123");
    }
}
