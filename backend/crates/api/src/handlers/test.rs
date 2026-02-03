use crate::handlers::auth::ErrorResponse;
use axum::{http::StatusCode, Json};
use ciam_auth::{EmailMessage, EmailService, SmsMessage, SmsService};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TestEmailRequest {
    pub to: String,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TestSmsRequest {
    pub to: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct TestResponse {
    pub success: bool,
    pub message: String,
}

/// Test email service
/// POST /api/test/email
pub async fn test_email(
    Json(request): Json<TestEmailRequest>,
) -> Result<Json<TestResponse>, (StatusCode, Json<ErrorResponse>)> {
    let email_service = EmailService::from_env().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("email_config_error", &e.to_string())),
        )
    })?;

    let email = EmailMessage {
        to: request.to.clone(),
        to_name: Some("Test User".to_string()),
        subject: request.subject,
        text_body: request.message.clone(),
        html_body: Some(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; padding: 20px; }}
        .container {{ max-width: 600px; margin: 0 auto; background: #f5f5f5; padding: 20px; border-radius: 8px; }}
        .message {{ background: white; padding: 20px; border-radius: 4px; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Test Email</h2>
        <div class="message">
            <p>{}</p>
        </div>
        <p style="color: #666; font-size: 12px; margin-top: 20px;">This is a test email from the CIAM system.</p>
    </div>
</body>
</html>"#,
            request.message.replace('\n', "<br>")
        )),
    };

    email_service.send(email).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("email_send_error", &e.to_string())),
        )
    })?;

    Ok(Json(TestResponse {
        success: true,
        message: format!("Test email sent to {}", request.to),
    }))
}

/// Test SMS service
/// POST /api/test/sms
pub async fn test_sms(
    Json(request): Json<TestSmsRequest>,
) -> Result<Json<TestResponse>, (StatusCode, Json<ErrorResponse>)> {
    let sms_service = SmsService::from_env().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("sms_config_error", &e.to_string())),
        )
    })?;

    let sms = SmsMessage {
        to: request.to.clone(),
        message: request.message,
    };

    sms_service.send(sms).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("sms_send_error", &e.to_string())),
        )
    })?;

    Ok(Json(TestResponse {
        success: true,
        message: format!("Test SMS sent to {}", request.to),
    }))
}

/// Test both email and SMS services connectivity
/// GET /api/test/connectivity
pub async fn test_connectivity() -> Result<Json<TestResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Test email service
    let email_service = EmailService::from_env().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("email_config_error", &e.to_string())),
        )
    })?;

    email_service.test_connection().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                "email_connection_error",
                &e.to_string(),
            )),
        )
    })?;

    // Test SMS service
    let sms_service = SmsService::from_env().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("sms_config_error", &e.to_string())),
        )
    })?;

    sms_service.test_connection().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("sms_connection_error", &e.to_string())),
        )
    })?;

    Ok(Json(TestResponse {
        success: true,
        message: "Email and SMS services are connected and ready".to_string(),
    }))
}
