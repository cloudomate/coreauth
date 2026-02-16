use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Webhook configuration for an organization
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub url: String,
    #[sqlx(skip)]
    #[serde(skip_serializing)]
    pub secret: String,
    pub events: Vec<String>,
    pub is_enabled: bool,
    pub retry_policy: sqlx::types::Json<RetryPolicy>,
    pub custom_headers: sqlx::types::Json<serde_json::Value>,
    pub total_deliveries: i32,
    pub successful_deliveries: i32,
    pub failed_deliveries: i32,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: i32,
    pub initial_delay_ms: i64,
    pub max_delay_ms: i64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
        }
    }
}

/// Request to create a new webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhook {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub retry_policy: Option<RetryPolicy>,
    #[serde(default)]
    pub custom_headers: Option<serde_json::Value>,
}

/// Request to update a webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebhook {
    pub name: Option<String>,
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_enabled: Option<bool>,
    pub retry_policy: Option<RetryPolicy>,
    pub custom_headers: Option<serde_json::Value>,
}

/// Webhook delivery record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_id: String,
    pub event_type: String,
    pub payload: sqlx::types::Json<serde_json::Value>,
    pub status: String,
    pub request_headers: Option<sqlx::types::Json<serde_json::Value>>,
    pub request_body: Option<String>,
    pub response_status: Option<i32>,
    pub response_headers: Option<sqlx::types::Json<serde_json::Value>>,
    pub response_body: Option<String>,
    pub response_time_ms: Option<i32>,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Success,
    Failed,
    Retrying,
}

impl std::fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryStatus::Pending => write!(f, "pending"),
            DeliveryStatus::Success => write!(f, "success"),
            DeliveryStatus::Failed => write!(f, "failed"),
            DeliveryStatus::Retrying => write!(f, "retrying"),
        }
    }
}

/// Webhook event stored in the event queue
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub event_id: String,
    pub organization_id: Uuid,
    pub event_type: String,
    pub payload: sqlx::types::Json<serde_json::Value>,
    pub status: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Supported webhook event type
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookEventType {
    pub id: String,
    pub category: String,
    pub description: String,
    pub payload_schema: Option<sqlx::types::Json<serde_json::Value>>,
    pub created_at: DateTime<Utc>,
}

/// Webhook event payload format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub organization_id: Uuid,
    pub data: serde_json::Value,
}

impl WebhookPayload {
    /// Create a new webhook payload
    pub fn new(event_type: &str, organization_id: Uuid, data: serde_json::Value) -> Self {
        Self {
            id: format!("evt_{}", Uuid::new_v4().to_string().replace("-", "")),
            event_type: event_type.to_string(),
            timestamp: Utc::now(),
            organization_id,
            data,
        }
    }
}

/// Response when creating/listing webhooks (without secret)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub is_enabled: bool,
    pub retry_policy: RetryPolicy,
    pub custom_headers: serde_json::Value,
    pub total_deliveries: i32,
    pub successful_deliveries: i32,
    pub failed_deliveries: i32,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Webhook> for WebhookResponse {
    fn from(w: Webhook) -> Self {
        Self {
            id: w.id,
            organization_id: w.organization_id,
            name: w.name,
            url: w.url,
            events: w.events,
            is_enabled: w.is_enabled,
            retry_policy: w.retry_policy.0,
            custom_headers: w.custom_headers.0,
            total_deliveries: w.total_deliveries,
            successful_deliveries: w.successful_deliveries,
            failed_deliveries: w.failed_deliveries,
            last_triggered_at: w.last_triggered_at,
            last_success_at: w.last_success_at,
            last_failure_at: w.last_failure_at,
            created_at: w.created_at,
            updated_at: w.updated_at,
        }
    }
}

/// Response when creating a webhook (includes secret once)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookWithSecretResponse {
    #[serde(flatten)]
    pub webhook: WebhookResponse,
    pub secret: String,
}

/// Test webhook request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWebhookRequest {
    pub event_type: Option<String>,
}

/// Test webhook response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWebhookResponse {
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<i64>,
    pub response_body: Option<String>,
    pub error: Option<String>,
}

/// Webhook delivery summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverySummary {
    pub id: Uuid,
    pub event_id: String,
    pub event_type: String,
    pub status: String,
    pub response_status: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub attempt_count: i32,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

impl From<WebhookDelivery> for DeliverySummary {
    fn from(d: WebhookDelivery) -> Self {
        Self {
            id: d.id,
            event_id: d.event_id,
            event_type: d.event_type,
            status: d.status,
            response_status: d.response_status,
            response_time_ms: d.response_time_ms,
            attempt_count: d.attempt_count,
            created_at: d.created_at,
            delivered_at: d.delivered_at,
        }
    }
}

/// Query parameters for listing deliveries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryQuery {
    pub webhook_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
