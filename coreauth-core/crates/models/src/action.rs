use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// JavaScript action triggered on specific events
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Action {
    pub id: Uuid,
    pub organization_id: Uuid,

    pub name: String,
    pub description: Option<String>,

    pub trigger_type: ActionTrigger,
    pub code: String,

    pub runtime: String,
    pub timeout_seconds: i32,

    #[sqlx(json)]
    pub secrets: serde_json::Value,

    pub execution_order: i32,
    pub is_enabled: bool,

    pub last_executed_at: Option<DateTime<Utc>>,
    pub total_executions: i64,
    pub total_failures: i64,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create new action request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAction {
    pub organization_id: Uuid,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    pub description: Option<String>,

    pub trigger_type: ActionTrigger,

    #[validate(length(min = 1))]
    pub code: String,

    pub runtime: Option<String>,

    #[validate(range(min = 1, max = 30))]
    pub timeout_seconds: Option<i32>,

    pub secrets: Option<serde_json::Value>,

    pub execution_order: Option<i32>,
}

/// Update action request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateAction {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    pub description: Option<String>,

    pub code: Option<String>,

    pub runtime: Option<String>,

    #[validate(range(min = 1, max = 30))]
    pub timeout_seconds: Option<i32>,

    pub secrets: Option<serde_json::Value>,

    pub execution_order: Option<i32>,

    pub is_enabled: Option<bool>,
}

/// Event trigger types for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionTrigger {
    PreLogin,
    PostLogin,
    PreRegistration,
    PostRegistration,
    PreTokenIssue,
    PostTokenIssue,
    PreUserUpdate,
    PostUserUpdate,
    PrePasswordReset,
    PostPasswordReset,
}

impl std::fmt::Display for ActionTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionTrigger::PreLogin => write!(f, "pre_login"),
            ActionTrigger::PostLogin => write!(f, "post_login"),
            ActionTrigger::PreRegistration => write!(f, "pre_registration"),
            ActionTrigger::PostRegistration => write!(f, "post_registration"),
            ActionTrigger::PreTokenIssue => write!(f, "pre_token_issue"),
            ActionTrigger::PostTokenIssue => write!(f, "post_token_issue"),
            ActionTrigger::PreUserUpdate => write!(f, "pre_user_update"),
            ActionTrigger::PostUserUpdate => write!(f, "post_user_update"),
            ActionTrigger::PrePasswordReset => write!(f, "pre_password_reset"),
            ActionTrigger::PostPasswordReset => write!(f, "post_password_reset"),
        }
    }
}

// SQLx implementation for ActionTrigger
impl sqlx::Type<sqlx::Postgres> for ActionTrigger {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("TEXT")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ActionTrigger {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "pre_login" => Ok(ActionTrigger::PreLogin),
            "post_login" => Ok(ActionTrigger::PostLogin),
            "pre_registration" => Ok(ActionTrigger::PreRegistration),
            "post_registration" => Ok(ActionTrigger::PostRegistration),
            "pre_token_issue" => Ok(ActionTrigger::PreTokenIssue),
            "post_token_issue" => Ok(ActionTrigger::PostTokenIssue),
            "pre_user_update" => Ok(ActionTrigger::PreUserUpdate),
            "post_user_update" => Ok(ActionTrigger::PostUserUpdate),
            "pre_password_reset" => Ok(ActionTrigger::PrePasswordReset),
            "post_password_reset" => Ok(ActionTrigger::PostPasswordReset),
            _ => Err(format!("Invalid action trigger: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ActionTrigger {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.to_string();
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s.as_str(), buf)
    }
}

/// Action execution result
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActionExecution {
    pub id: Uuid,
    pub action_id: Uuid,
    pub organization_id: Uuid,

    pub trigger_type: String,
    pub user_id: Option<Uuid>,

    pub status: ExecutionStatus,
    pub execution_time_ms: i32,

    #[sqlx(json)]
    pub input_data: Option<serde_json::Value>,

    #[sqlx(json)]
    pub output_data: Option<serde_json::Value>,

    pub error_message: Option<String>,

    pub executed_at: DateTime<Utc>,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Success,
    Failure,
    Timeout,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Success => write!(f, "success"),
            ExecutionStatus::Failure => write!(f, "failure"),
            ExecutionStatus::Timeout => write!(f, "timeout"),
        }
    }
}

// SQLx implementation for ExecutionStatus
impl sqlx::Type<sqlx::Postgres> for ExecutionStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("TEXT")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ExecutionStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "success" => Ok(ExecutionStatus::Success),
            "failure" => Ok(ExecutionStatus::Failure),
            "timeout" => Ok(ExecutionStatus::Timeout),
            _ => Err(format!("Invalid execution status: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ExecutionStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.to_string();
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s.as_str(), buf)
    }
}

/// Context passed to action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContext {
    pub user: Option<serde_json::Value>,
    pub organization: Option<serde_json::Value>,
    pub event: String,
    pub metadata: serde_json::Value,
}

/// Result from action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}
