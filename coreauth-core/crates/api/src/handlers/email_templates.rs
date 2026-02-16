use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use ciam_auth::email::templates::{
    CustomEmailTemplate, EmailBranding, EmailTemplateType, fetch_custom_template,
    render_email,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::ErrorResponse;
use crate::AppState;

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

// ============================================================================
// Request/Response types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct EmailTemplateListItem {
    pub template_type: String,
    pub description: String,
    pub has_custom_template: bool,
    pub available_variables: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EmailTemplateResponse {
    pub template_type: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
    pub is_custom: bool,
    pub available_variables: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmailTemplateRequest {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

#[derive(Debug, Serialize)]
pub struct PreviewEmailTemplateResponse {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

fn template_description(t: EmailTemplateType) -> &'static str {
    match t {
        EmailTemplateType::EmailVerification => "Sent when a user needs to verify their email address",
        EmailTemplateType::PasswordReset => "Sent when a user requests a password reset",
        EmailTemplateType::UserInvitation => "Sent when a user is invited to join the organization",
        EmailTemplateType::MagicLink => "Sent for passwordless login via magic link",
        EmailTemplateType::AccountLocked => "Sent when a user's account is temporarily locked",
        EmailTemplateType::MfaEnforcement => "Sent when MFA is required and user needs to set it up",
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/organizations/:org_id/email-templates
/// List all 6 template types with custom/default status
pub async fn list_email_templates(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<EmailTemplateListItem>>, (StatusCode, Json<ErrorResponse>)> {
    let pool = state.auth_service.db.pool();

    // Get all custom templates for this tenant
    let custom_types: Vec<(String,)> = sqlx::query_as(
        "SELECT template_type FROM email_templates WHERE tenant_id = $1 AND is_active = true",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", e.to_string()))?;

    let custom_set: std::collections::HashSet<String> =
        custom_types.into_iter().map(|(t,)| t).collect();

    let items: Vec<EmailTemplateListItem> = EmailTemplateType::all()
        .iter()
        .map(|t| EmailTemplateListItem {
            template_type: t.as_str().to_string(),
            description: template_description(*t).to_string(),
            has_custom_template: custom_set.contains(t.as_str()),
            available_variables: t.available_variables().into_iter().map(|s| s.to_string()).collect(),
        })
        .collect();

    Ok(Json(items))
}

/// GET /api/organizations/:org_id/email-templates/:template_type
/// Get template (custom if exists, else built-in default)
pub async fn get_email_template(
    State(state): State<Arc<AppState>>,
    Path((org_id, template_type_str)): Path<(Uuid, String)>,
) -> Result<Json<EmailTemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let template_type = EmailTemplateType::from_str(&template_type_str)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "invalid_template_type",
            format!("Invalid template type: {}. Valid types: {:?}",
                template_type_str,
                EmailTemplateType::all().iter().map(|t| t.as_str()).collect::<Vec<_>>())))?;

    let pool = state.auth_service.db.pool();
    let custom = fetch_custom_template(pool, org_id, template_type).await;

    let branding = get_tenant_branding(pool, org_id).await;
    let variables = template_type.sample_variables();

    let (is_custom, subject, text_body, html_body) = if let Some(ref custom_tpl) = custom {
        (true, custom_tpl.subject.clone(), custom_tpl.text_body.clone(), custom_tpl.html_body.clone())
    } else {
        // Return the built-in default rendered with sample data
        let (subj, text, html) = render_email(template_type, None, &variables, &branding);
        (false, subj, text, html)
    };

    Ok(Json(EmailTemplateResponse {
        template_type: template_type_str,
        subject,
        html_body,
        text_body,
        is_custom,
        available_variables: template_type.available_variables().into_iter().map(|s| s.to_string()).collect(),
    }))
}

/// PUT /api/organizations/:org_id/email-templates/:template_type
/// Upsert custom template
pub async fn update_email_template(
    State(state): State<Arc<AppState>>,
    Path((org_id, template_type_str)): Path<(Uuid, String)>,
    Json(body): Json<UpdateEmailTemplateRequest>,
) -> Result<Json<EmailTemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let template_type = EmailTemplateType::from_str(&template_type_str)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "invalid_template_type",
            format!("Invalid template type: {}", template_type_str)))?;

    let pool = state.auth_service.db.pool();

    // Upsert the template
    sqlx::query(
        r#"
        INSERT INTO email_templates (tenant_id, template_type, subject, html_body, text_body)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (tenant_id, template_type)
        DO UPDATE SET subject = $3, html_body = $4, text_body = $5, is_active = true, updated_at = NOW()
        "#,
    )
    .bind(org_id)
    .bind(template_type.as_str())
    .bind(&body.subject)
    .bind(&body.html_body)
    .bind(&body.text_body)
    .execute(pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", e.to_string()))?;

    Ok(Json(EmailTemplateResponse {
        template_type: template_type_str,
        subject: body.subject,
        html_body: body.html_body,
        text_body: body.text_body,
        is_custom: true,
        available_variables: template_type.available_variables().into_iter().map(|s| s.to_string()).collect(),
    }))
}

/// DELETE /api/organizations/:org_id/email-templates/:template_type
/// Reset to default (delete custom override)
pub async fn delete_email_template(
    State(state): State<Arc<AppState>>,
    Path((org_id, template_type_str)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let template_type = EmailTemplateType::from_str(&template_type_str)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "invalid_template_type",
            format!("Invalid template type: {}", template_type_str)))?;

    let pool = state.auth_service.db.pool();

    sqlx::query(
        "DELETE FROM email_templates WHERE tenant_id = $1 AND template_type = $2",
    )
    .bind(org_id)
    .bind(template_type.as_str())
    .execute(pool)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "database_error", e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/organizations/:org_id/email-templates/:template_type/preview
/// Render template with sample data
pub async fn preview_email_template(
    State(state): State<Arc<AppState>>,
    Path((org_id, template_type_str)): Path<(Uuid, String)>,
    body: Option<Json<UpdateEmailTemplateRequest>>,
) -> Result<Json<PreviewEmailTemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let template_type = EmailTemplateType::from_str(&template_type_str)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "invalid_template_type",
            format!("Invalid template type: {}", template_type_str)))?;

    let pool = state.auth_service.db.pool();
    let branding = get_tenant_branding(pool, org_id).await;
    let variables = template_type.sample_variables();

    let (subject, text_body, html_body) = if let Some(Json(tpl)) = body {
        // Preview the provided template body
        let custom = CustomEmailTemplate {
            subject: tpl.subject,
            html_body: tpl.html_body,
            text_body: tpl.text_body,
        };
        render_email(template_type, Some(&custom), &variables, &branding)
    } else {
        // Preview the currently saved template (custom or default)
        let custom = fetch_custom_template(pool, org_id, template_type).await;
        render_email(template_type, custom.as_ref(), &variables, &branding)
    };

    Ok(Json(PreviewEmailTemplateResponse {
        subject,
        html_body,
        text_body,
    }))
}

// ============================================================================
// Helpers
// ============================================================================

async fn get_tenant_branding(pool: &sqlx::PgPool, tenant_id: Uuid) -> EmailBranding {
    let settings: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT settings FROM tenants WHERE id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    settings
        .and_then(|(v,)| serde_json::from_value::<ciam_models::OrganizationSettings>(v).ok())
        .map(|s| EmailBranding::from_settings(&s.branding))
        .unwrap_or_default()
}
