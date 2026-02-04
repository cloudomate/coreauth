use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use ciam_models::{
    CreateWebhook, DeliveryQuery, DeliverySummary, RetryPolicy, TestWebhookRequest,
    TestWebhookResponse, UpdateWebhook, WebhookDelivery, WebhookEventType, WebhookPayload,
    WebhookResponse, WebhookWithSecretResponse,
};
use hmac::{Hmac, Mac};
use rand::Rng;
use reqwest::Client;
use sha2::Sha256;
use sqlx::types::Json as SqlxJson;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::handlers::ErrorResponse;
use crate::AppState;

type HmacSha256 = Hmac<Sha256>;

/// Helper to create error responses
fn err(status: StatusCode, code: &str, msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: code.to_string(),
            message: msg.into(),
        }),
    )
}

/// List all webhooks for an organization
pub async fn list_webhooks(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let webhooks: Vec<WebhookRow> = sqlx::query_as(
        r#"
        SELECT id, organization_id, name, url, events, is_enabled,
               retry_policy, custom_headers,
               total_deliveries, successful_deliveries, failed_deliveries,
               last_triggered_at, last_success_at, last_failure_at,
               created_at, updated_at
        FROM webhooks
        WHERE organization_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(org_id)
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to list webhooks: {}", e)))?;

    let responses: Vec<WebhookResponse> = webhooks.into_iter().map(|w| w.into()).collect();
    Ok(Json(responses))
}

/// Get a single webhook
pub async fn get_webhook(
    State(state): State<Arc<AppState>>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let webhook: WebhookRow = sqlx::query_as(
        r#"
        SELECT id, organization_id, name, url, events, is_enabled,
               retry_policy, custom_headers,
               total_deliveries, successful_deliveries, failed_deliveries,
               last_triggered_at, last_success_at, last_failure_at,
               created_at, updated_at
        FROM webhooks
        WHERE id = $1 AND organization_id = $2
        "#,
    )
    .bind(webhook_id)
    .bind(org_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to get webhook: {}", e)))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "not_found", "Webhook not found"))?;

    let response: WebhookResponse = webhook.into();
    Ok(Json(response))
}

/// Create a new webhook
pub async fn create_webhook(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateWebhook>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Validate URL
    if !req.url.starts_with("https://") && !req.url.starts_with("http://localhost") {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "invalid_url",
            "Webhook URL must use HTTPS (except localhost for development)",
        ));
    }

    // Validate events
    if req.events.is_empty() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "invalid_events",
            "At least one event type must be specified",
        ));
    }

    // Generate secret
    let secret = generate_webhook_secret();
    let retry_policy = req.retry_policy.unwrap_or_default();
    let custom_headers = req.custom_headers.unwrap_or(serde_json::json!({}));

    let webhook: WebhookRowWithSecret = sqlx::query_as(
        r#"
        INSERT INTO webhooks (organization_id, name, url, secret, events, is_enabled, retry_policy, custom_headers)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, organization_id, name, url, secret, events, is_enabled,
                  retry_policy, custom_headers,
                  total_deliveries, successful_deliveries, failed_deliveries,
                  last_triggered_at, last_success_at, last_failure_at,
                  created_at, updated_at
        "#,
    )
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.url)
    .bind(&secret)
    .bind(&req.events)
    .bind(req.is_enabled)
    .bind(SqlxJson(retry_policy))
    .bind(SqlxJson(custom_headers))
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to create webhook: {}", e)))?;

    let response = WebhookWithSecretResponse {
        webhook: webhook.clone().into(),
        secret: webhook.secret,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Update a webhook
pub async fn update_webhook(
    State(state): State<Arc<AppState>>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateWebhook>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Validate URL if provided
    if let Some(ref url) = req.url {
        if !url.starts_with("https://") && !url.starts_with("http://localhost") {
            return Err(err(
                StatusCode::BAD_REQUEST,
                "invalid_url",
                "Webhook URL must use HTTPS (except localhost for development)",
            ));
        }
    }

    // Validate events if provided
    if let Some(ref events) = req.events {
        if events.is_empty() {
            return Err(err(
                StatusCode::BAD_REQUEST,
                "invalid_events",
                "At least one event type must be specified",
            ));
        }
    }

    // Build update query dynamically
    let webhook: WebhookRow = sqlx::query_as(
        r#"
        UPDATE webhooks SET
            name = COALESCE($3, name),
            url = COALESCE($4, url),
            events = COALESCE($5, events),
            is_enabled = COALESCE($6, is_enabled),
            retry_policy = COALESCE($7, retry_policy),
            custom_headers = COALESCE($8, custom_headers),
            updated_at = NOW()
        WHERE id = $1 AND organization_id = $2
        RETURNING id, organization_id, name, url, events, is_enabled,
                  retry_policy, custom_headers,
                  total_deliveries, successful_deliveries, failed_deliveries,
                  last_triggered_at, last_success_at, last_failure_at,
                  created_at, updated_at
        "#,
    )
    .bind(webhook_id)
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.url)
    .bind(&req.events)
    .bind(&req.is_enabled)
    .bind(req.retry_policy.map(SqlxJson))
    .bind(req.custom_headers.map(SqlxJson))
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to update webhook: {}", e)))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "not_found", "Webhook not found"))?;

    let response: WebhookResponse = webhook.into();
    Ok(Json(response))
}

/// Delete a webhook
pub async fn delete_webhook(
    State(state): State<Arc<AppState>>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query(
        "DELETE FROM webhooks WHERE id = $1 AND organization_id = $2"
    )
    .bind(webhook_id)
    .bind(org_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to delete webhook: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "not_found", "Webhook not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Rotate webhook secret
pub async fn rotate_secret(
    State(state): State<Arc<AppState>>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let new_secret = generate_webhook_secret();

    let result = sqlx::query(
        r#"
        UPDATE webhooks SET secret = $3, updated_at = NOW()
        WHERE id = $1 AND organization_id = $2
        "#,
    )
    .bind(webhook_id)
    .bind(org_id)
    .bind(&new_secret)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to rotate secret: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "not_found", "Webhook not found"));
    }

    Ok(Json(serde_json::json!({ "secret": new_secret })))
}

/// Test a webhook with a sample event
pub async fn test_webhook(
    State(state): State<Arc<AppState>>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<TestWebhookRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Get webhook
    let webhook: WebhookRowWithSecret = sqlx::query_as(
        r#"
        SELECT id, organization_id, name, url, secret, events, is_enabled,
               retry_policy, custom_headers,
               total_deliveries, successful_deliveries, failed_deliveries,
               last_triggered_at, last_success_at, last_failure_at,
               created_at, updated_at
        FROM webhooks
        WHERE id = $1 AND organization_id = $2
        "#,
    )
    .bind(webhook_id)
    .bind(org_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to get webhook: {}", e)))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "not_found", "Webhook not found"))?;

    // Create test payload
    let event_type = req.event_type.unwrap_or_else(|| "test.webhook".to_string());
    let payload = WebhookPayload::new(
        &event_type,
        org_id,
        serde_json::json!({
            "message": "This is a test webhook delivery",
            "webhook_id": webhook_id,
            "timestamp": Utc::now().to_rfc3339(),
        }),
    );

    // Send test request
    let result = send_webhook_request(&webhook.url, &webhook.secret, &payload, &webhook.custom_headers).await;

    let response = match result {
        Ok((status, body, duration_ms)) => TestWebhookResponse {
            success: status.is_success(),
            status_code: Some(status.as_u16()),
            response_time_ms: Some(duration_ms),
            response_body: Some(body),
            error: None,
        },
        Err(e) => TestWebhookResponse {
            success: false,
            status_code: None,
            response_time_ms: None,
            response_body: None,
            error: Some(e),
        },
    };

    Ok(Json(response))
}

/// List webhook deliveries
pub async fn list_deliveries(
    State(state): State<Arc<AppState>>,
    Path((org_id, webhook_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<DeliveryQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Verify webhook belongs to org
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM webhooks WHERE id = $1 AND organization_id = $2)"
    )
    .bind(webhook_id)
    .bind(org_id)
    .fetch_one(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Database error: {}", e)))?;

    if !exists {
        return Err(err(StatusCode::NOT_FOUND, "not_found", "Webhook not found"));
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let deliveries: Vec<WebhookDelivery> = sqlx::query_as(
        r#"
        SELECT id, webhook_id, event_id, event_type, payload, status,
               request_headers, request_body, response_status, response_headers,
               response_body, response_time_ms, attempt_count, max_attempts,
               next_retry_at, last_error, delivered_at, failed_at, created_at
        FROM webhook_deliveries
        WHERE webhook_id = $1
        AND ($2::text IS NULL OR event_type = $2)
        AND ($3::text IS NULL OR status = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(webhook_id)
    .bind(&query.event_type)
    .bind(&query.status)
    .bind(limit)
    .bind(offset)
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to list deliveries: {}", e)))?;

    let summaries: Vec<DeliverySummary> = deliveries.into_iter().map(|d| d.into()).collect();
    Ok(Json(summaries))
}

/// Get a single delivery with full details
pub async fn get_delivery(
    State(state): State<Arc<AppState>>,
    Path((org_id, webhook_id, delivery_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let delivery: WebhookDelivery = sqlx::query_as(
        r#"
        SELECT d.id, d.webhook_id, d.event_id, d.event_type, d.payload, d.status,
               d.request_headers, d.request_body, d.response_status, d.response_headers,
               d.response_body, d.response_time_ms, d.attempt_count, d.max_attempts,
               d.next_retry_at, d.last_error, d.delivered_at, d.failed_at, d.created_at
        FROM webhook_deliveries d
        JOIN webhooks w ON w.id = d.webhook_id
        WHERE d.id = $1 AND d.webhook_id = $2 AND w.organization_id = $3
        "#,
    )
    .bind(delivery_id)
    .bind(webhook_id)
    .bind(org_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to get delivery: {}", e)))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "not_found", "Delivery not found"))?;

    Ok(Json(delivery))
}

/// Retry a failed delivery
pub async fn retry_delivery(
    State(state): State<Arc<AppState>>,
    Path((org_id, webhook_id, delivery_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Get delivery and webhook
    let _delivery: WebhookDelivery = sqlx::query_as(
        r#"
        SELECT d.id, d.webhook_id, d.event_id, d.event_type, d.payload, d.status,
               d.request_headers, d.request_body, d.response_status, d.response_headers,
               d.response_body, d.response_time_ms, d.attempt_count, d.max_attempts,
               d.next_retry_at, d.last_error, d.delivered_at, d.failed_at, d.created_at
        FROM webhook_deliveries d
        JOIN webhooks w ON w.id = d.webhook_id
        WHERE d.id = $1 AND d.webhook_id = $2 AND w.organization_id = $3
        "#,
    )
    .bind(delivery_id)
    .bind(webhook_id)
    .bind(org_id)
    .fetch_optional(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Database error: {}", e)))?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "not_found", "Delivery not found"))?;

    // Mark for immediate retry
    sqlx::query(
        r#"
        UPDATE webhook_deliveries
        SET status = 'retrying', next_retry_at = NOW(), attempt_count = attempt_count
        WHERE id = $1
        "#,
    )
    .bind(delivery_id)
    .execute(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to schedule retry: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Delivery scheduled for retry",
        "delivery_id": delivery_id,
    })))
}

/// List available event types
pub async fn list_event_types(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let event_types: Vec<WebhookEventType> = sqlx::query_as(
        "SELECT id, category, description, payload_schema, created_at FROM webhook_event_types ORDER BY category, id"
    )
    .fetch_all(state.auth_service.db.pool())
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", format!("Failed to list event types: {}", e)))?;

    Ok(Json(event_types))
}

// ============================================================================
// Helper Types and Functions
// ============================================================================

/// Internal row type for webhook queries (without secret)
#[derive(Debug, Clone, sqlx::FromRow)]
struct WebhookRow {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    url: String,
    events: Vec<String>,
    is_enabled: bool,
    retry_policy: SqlxJson<RetryPolicy>,
    custom_headers: SqlxJson<serde_json::Value>,
    total_deliveries: i32,
    successful_deliveries: i32,
    failed_deliveries: i32,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    last_success_at: Option<chrono::DateTime<Utc>>,
    last_failure_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<WebhookRow> for WebhookResponse {
    fn from(w: WebhookRow) -> Self {
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

/// Internal row type for webhook queries (with secret)
#[derive(Debug, Clone, sqlx::FromRow)]
struct WebhookRowWithSecret {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    url: String,
    secret: String,
    events: Vec<String>,
    is_enabled: bool,
    retry_policy: SqlxJson<RetryPolicy>,
    custom_headers: SqlxJson<serde_json::Value>,
    total_deliveries: i32,
    successful_deliveries: i32,
    failed_deliveries: i32,
    last_triggered_at: Option<chrono::DateTime<Utc>>,
    last_success_at: Option<chrono::DateTime<Utc>>,
    last_failure_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<WebhookRowWithSecret> for WebhookResponse {
    fn from(w: WebhookRowWithSecret) -> Self {
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

/// Generate a secure webhook secret
fn generate_webhook_secret() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    format!("whsec_{}", hex::encode(bytes))
}

/// Sign a webhook payload using HMAC-SHA256
pub fn sign_payload(secret: &str, timestamp: i64, payload: &str) -> String {
    let message = format!("{}.{}", timestamp, payload);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Send a webhook request
async fn send_webhook_request(
    url: &str,
    secret: &str,
    payload: &WebhookPayload,
    custom_headers: &SqlxJson<serde_json::Value>,
) -> Result<(reqwest::StatusCode, String, i64), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let payload_json = serde_json::to_string(payload)
        .map_err(|e| format!("Failed to serialize payload: {}", e))?;

    let timestamp = Utc::now().timestamp();
    let signature = sign_payload(secret, timestamp, &payload_json);

    let start = Instant::now();

    let mut request = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-CoreAuth-Signature", format!("sha256={}", signature))
        .header("X-CoreAuth-Timestamp", timestamp.to_string())
        .header("X-CoreAuth-Event-ID", &payload.id)
        .header("X-CoreAuth-Event-Type", &payload.event_type);

    // Add custom headers
    if let Some(headers) = custom_headers.0.as_object() {
        for (key, value) in headers {
            if let Some(v) = value.as_str() {
                request = request.header(key, v);
            }
        }
    }

    let response = request
        .body(payload_json)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let duration_ms = start.elapsed().as_millis() as i64;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    Ok((status, body, duration_ms))
}

// ============================================================================
// Event Publishing Service
// ============================================================================

/// Publish a webhook event to all subscribed webhooks
pub async fn publish_event(
    pool: &sqlx::PgPool,
    organization_id: Uuid,
    event_type: &str,
    data: serde_json::Value,
) -> Result<(), String> {
    let payload = WebhookPayload::new(event_type, organization_id, data);

    // Get all enabled webhooks for this org that subscribe to this event
    let webhooks: Vec<WebhookRowWithSecret> = sqlx::query_as(
        r#"
        SELECT id, organization_id, name, url, secret, events, is_enabled,
               retry_policy, custom_headers,
               total_deliveries, successful_deliveries, failed_deliveries,
               last_triggered_at, last_success_at, last_failure_at,
               created_at, updated_at
        FROM webhooks
        WHERE organization_id = $1
        AND is_enabled = true
        AND $2 = ANY(events)
        "#,
    )
    .bind(organization_id)
    .bind(event_type)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to query webhooks: {}", e))?;

    // Create delivery records and dispatch
    for webhook in webhooks {
        let retry_policy = webhook.retry_policy.0.clone();

        // Create delivery record
        let delivery_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO webhook_deliveries (webhook_id, event_id, event_type, payload, max_attempts)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(webhook.id)
        .bind(&payload.id)
        .bind(&payload.event_type)
        .bind(SqlxJson(&payload))
        .bind(retry_policy.max_retries + 1)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to create delivery record: {}", e))?;

        // Attempt delivery
        let result = send_webhook_request(
            &webhook.url,
            &webhook.secret,
            &payload,
            &webhook.custom_headers,
        )
        .await;

        match result {
            Ok((status, body, duration_ms)) => {
                if status.is_success() {
                    // Mark as success
                    let _ = sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = 'success',
                            response_status = $2,
                            response_body = $3,
                            response_time_ms = $4,
                            attempt_count = 1,
                            delivered_at = NOW()
                        WHERE id = $1
                        "#,
                    )
                    .bind(delivery_id)
                    .bind(status.as_u16() as i32)
                    .bind(&body)
                    .bind(duration_ms as i32)
                    .execute(pool)
                    .await;
                } else {
                    // Schedule retry
                    let next_retry = Utc::now() + Duration::milliseconds(retry_policy.initial_delay_ms);
                    let _ = sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = 'retrying',
                            response_status = $2,
                            response_body = $3,
                            response_time_ms = $4,
                            attempt_count = 1,
                            next_retry_at = $5,
                            last_error = $6
                        WHERE id = $1
                        "#,
                    )
                    .bind(delivery_id)
                    .bind(status.as_u16() as i32)
                    .bind(&body)
                    .bind(duration_ms as i32)
                    .bind(next_retry)
                    .bind(format!("HTTP {}", status.as_u16()))
                    .execute(pool)
                    .await;
                }
            }
            Err(e) => {
                // Schedule retry
                let next_retry = Utc::now() + Duration::milliseconds(retry_policy.initial_delay_ms);
                let _ = sqlx::query(
                    r#"
                    UPDATE webhook_deliveries
                    SET status = 'retrying',
                        attempt_count = 1,
                        next_retry_at = $2,
                        last_error = $3
                    WHERE id = $1
                    "#,
                )
                .bind(delivery_id)
                .bind(next_retry)
                .bind(&e)
                .execute(pool)
                .await;
            }
        }
    }

    Ok(())
}

/// Process pending retries (should be called by a background worker)
pub async fn process_pending_retries(pool: &sqlx::PgPool) -> Result<i64, String> {
    // Get deliveries ready for retry
    let deliveries: Vec<(Uuid, Uuid, SqlxJson<serde_json::Value>, i32, i32)> = sqlx::query_as(
        r#"
        SELECT d.id, d.webhook_id, d.payload, d.attempt_count, d.max_attempts
        FROM webhook_deliveries d
        WHERE d.status = 'retrying'
        AND d.next_retry_at <= NOW()
        ORDER BY d.next_retry_at
        LIMIT 100
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to query pending retries: {}", e))?;

    let count = deliveries.len() as i64;

    for (delivery_id, webhook_id, payload, attempt_count, max_attempts) in deliveries {
        // Get webhook details
        let webhook: Option<WebhookRowWithSecret> = sqlx::query_as(
            r#"
            SELECT id, organization_id, name, url, secret, events, is_enabled,
                   retry_policy, custom_headers,
                   total_deliveries, successful_deliveries, failed_deliveries,
                   last_triggered_at, last_success_at, last_failure_at,
                   created_at, updated_at
            FROM webhooks
            WHERE id = $1 AND is_enabled = true
            "#,
        )
        .bind(webhook_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get webhook: {}", e))?;

        let Some(webhook) = webhook else {
            // Webhook disabled or deleted, mark as failed
            let _ = sqlx::query(
                "UPDATE webhook_deliveries SET status = 'failed', failed_at = NOW() WHERE id = $1"
            )
            .bind(delivery_id)
            .execute(pool)
            .await;
            continue;
        };

        let retry_policy = webhook.retry_policy.0.clone();
        let payload: WebhookPayload = serde_json::from_value(payload.0.clone())
            .map_err(|e| format!("Failed to deserialize payload: {}", e))?;

        // Attempt delivery
        let result = send_webhook_request(
            &webhook.url,
            &webhook.secret,
            &payload,
            &webhook.custom_headers,
        )
        .await;

        let new_attempt_count = attempt_count + 1;

        match result {
            Ok((status, body, duration_ms)) => {
                if status.is_success() {
                    let _ = sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = 'success',
                            response_status = $2,
                            response_body = $3,
                            response_time_ms = $4,
                            attempt_count = $5,
                            delivered_at = NOW(),
                            next_retry_at = NULL
                        WHERE id = $1
                        "#,
                    )
                    .bind(delivery_id)
                    .bind(status.as_u16() as i32)
                    .bind(&body)
                    .bind(duration_ms as i32)
                    .bind(new_attempt_count)
                    .execute(pool)
                    .await;
                } else if new_attempt_count >= max_attempts {
                    // Max retries exceeded
                    let _ = sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = 'failed',
                            response_status = $2,
                            response_body = $3,
                            response_time_ms = $4,
                            attempt_count = $5,
                            failed_at = NOW(),
                            next_retry_at = NULL,
                            last_error = $6
                        WHERE id = $1
                        "#,
                    )
                    .bind(delivery_id)
                    .bind(status.as_u16() as i32)
                    .bind(&body)
                    .bind(duration_ms as i32)
                    .bind(new_attempt_count)
                    .bind(format!("Max retries exceeded. Last status: HTTP {}", status.as_u16()))
                    .execute(pool)
                    .await;
                } else {
                    // Schedule next retry with exponential backoff
                    let delay = calculate_backoff(new_attempt_count, &retry_policy);
                    let next_retry = Utc::now() + Duration::milliseconds(delay);
                    let _ = sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET response_status = $2,
                            response_body = $3,
                            response_time_ms = $4,
                            attempt_count = $5,
                            next_retry_at = $6,
                            last_error = $7
                        WHERE id = $1
                        "#,
                    )
                    .bind(delivery_id)
                    .bind(status.as_u16() as i32)
                    .bind(&body)
                    .bind(duration_ms as i32)
                    .bind(new_attempt_count)
                    .bind(next_retry)
                    .bind(format!("HTTP {}", status.as_u16()))
                    .execute(pool)
                    .await;
                }
            }
            Err(e) => {
                if new_attempt_count >= max_attempts {
                    let _ = sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = 'failed',
                            attempt_count = $2,
                            failed_at = NOW(),
                            next_retry_at = NULL,
                            last_error = $3
                        WHERE id = $1
                        "#,
                    )
                    .bind(delivery_id)
                    .bind(new_attempt_count)
                    .bind(format!("Max retries exceeded. Last error: {}", e))
                    .execute(pool)
                    .await;
                } else {
                    let delay = calculate_backoff(new_attempt_count, &retry_policy);
                    let next_retry = Utc::now() + Duration::milliseconds(delay);
                    let _ = sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET attempt_count = $2,
                            next_retry_at = $3,
                            last_error = $4
                        WHERE id = $1
                        "#,
                    )
                    .bind(delivery_id)
                    .bind(new_attempt_count)
                    .bind(next_retry)
                    .bind(&e)
                    .execute(pool)
                    .await;
                }
            }
        }
    }

    Ok(count)
}

/// Calculate backoff delay with exponential backoff
fn calculate_backoff(attempt: i32, policy: &RetryPolicy) -> i64 {
    let delay = policy.initial_delay_ms * 2_i64.pow((attempt - 1) as u32);
    delay.min(policy.max_delay_ms)
}
