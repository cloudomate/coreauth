//! Webhook event publishing helpers
//!
//! This module provides convenient functions for publishing webhook events
//! from various parts of the application.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::handlers::webhook::publish_event;

/// Publish a user.created event
pub async fn publish_user_created(
    pool: &PgPool,
    organization_id: Uuid,
    user_id: Uuid,
    email: &str,
    name: Option<&str>,
) {
    let data = json!({
        "user": {
            "id": user_id,
            "email": email,
            "name": name,
        }
    });

    if let Err(e) = publish_event(pool, organization_id, "user.created", data).await {
        tracing::error!("Failed to publish user.created event: {}", e);
    }
}

/// Publish a user.updated event
pub async fn publish_user_updated(
    pool: &PgPool,
    organization_id: Uuid,
    user_id: Uuid,
    changed_fields: Vec<&str>,
) {
    let data = json!({
        "user": {
            "id": user_id,
        },
        "changed_fields": changed_fields,
    });

    if let Err(e) = publish_event(pool, organization_id, "user.updated", data).await {
        tracing::error!("Failed to publish user.updated event: {}", e);
    }
}

/// Publish a user.deleted event
pub async fn publish_user_deleted(pool: &PgPool, organization_id: Uuid, user_id: Uuid) {
    let data = json!({
        "user_id": user_id,
    });

    if let Err(e) = publish_event(pool, organization_id, "user.deleted", data).await {
        tracing::error!("Failed to publish user.deleted event: {}", e);
    }
}

/// Publish a user.login event
pub async fn publish_user_login(
    pool: &PgPool,
    organization_id: Uuid,
    user_id: Uuid,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    connection_type: Option<&str>,
) {
    let data = json!({
        "user_id": user_id,
        "ip_address": ip_address,
        "user_agent": user_agent,
        "connection_type": connection_type,
    });

    if let Err(e) = publish_event(pool, organization_id, "user.login", data).await {
        tracing::error!("Failed to publish user.login event: {}", e);
    }
}

/// Publish a user.login_failed event
pub async fn publish_user_login_failed(
    pool: &PgPool,
    organization_id: Uuid,
    email: &str,
    ip_address: Option<&str>,
    reason: &str,
) {
    let data = json!({
        "email": email,
        "ip_address": ip_address,
        "reason": reason,
    });

    if let Err(e) = publish_event(pool, organization_id, "user.login_failed", data).await {
        tracing::error!("Failed to publish user.login_failed event: {}", e);
    }
}

/// Publish a user.logout event
pub async fn publish_user_logout(pool: &PgPool, organization_id: Uuid, user_id: Uuid) {
    let data = json!({
        "user_id": user_id,
    });

    if let Err(e) = publish_event(pool, organization_id, "user.logout", data).await {
        tracing::error!("Failed to publish user.logout event: {}", e);
    }
}

/// Publish a user.password_changed event
pub async fn publish_user_password_changed(pool: &PgPool, organization_id: Uuid, user_id: Uuid) {
    let data = json!({
        "user_id": user_id,
    });

    if let Err(e) = publish_event(pool, organization_id, "user.password_changed", data).await {
        tracing::error!("Failed to publish user.password_changed event: {}", e);
    }
}

/// Publish a user.mfa_enrolled event
pub async fn publish_user_mfa_enrolled(
    pool: &PgPool,
    organization_id: Uuid,
    user_id: Uuid,
    method: &str,
) {
    let data = json!({
        "user_id": user_id,
        "method": method,
    });

    if let Err(e) = publish_event(pool, organization_id, "user.mfa_enrolled", data).await {
        tracing::error!("Failed to publish user.mfa_enrolled event: {}", e);
    }
}

/// Publish an organization.created event
pub async fn publish_organization_created(
    pool: &PgPool,
    organization_id: Uuid,
    name: &str,
    slug: &str,
) {
    let data = json!({
        "organization": {
            "id": organization_id,
            "name": name,
            "slug": slug,
        }
    });

    if let Err(e) = publish_event(pool, organization_id, "organization.created", data).await {
        tracing::error!("Failed to publish organization.created event: {}", e);
    }
}

/// Publish an organization.member_added event
pub async fn publish_organization_member_added(
    pool: &PgPool,
    organization_id: Uuid,
    user_id: Uuid,
    role: &str,
) {
    let data = json!({
        "organization_id": organization_id,
        "user_id": user_id,
        "role": role,
    });

    if let Err(e) = publish_event(pool, organization_id, "organization.member_added", data).await {
        tracing::error!("Failed to publish organization.member_added event: {}", e);
    }
}

/// Publish an application.created event
pub async fn publish_application_created(
    pool: &PgPool,
    organization_id: Uuid,
    application_id: Uuid,
    name: &str,
    client_id: &str,
) {
    let data = json!({
        "application": {
            "id": application_id,
            "name": name,
            "client_id": client_id,
        }
    });

    if let Err(e) = publish_event(pool, organization_id, "application.created", data).await {
        tracing::error!("Failed to publish application.created event: {}", e);
    }
}

/// Publish a connection.created event
pub async fn publish_connection_created(
    pool: &PgPool,
    organization_id: Uuid,
    connection_id: Uuid,
    name: &str,
    connection_type: &str,
) {
    let data = json!({
        "connection": {
            "id": connection_id,
            "name": name,
            "type": connection_type,
        }
    });

    if let Err(e) = publish_event(pool, organization_id, "connection.created", data).await {
        tracing::error!("Failed to publish connection.created event: {}", e);
    }
}
