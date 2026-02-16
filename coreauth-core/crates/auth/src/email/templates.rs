use chrono::{DateTime, Utc};
use std::collections::HashMap;

// ============================================================================
// TEMPLATE TYPES & CUSTOM TEMPLATE ENGINE
// ============================================================================

/// The six supported email template types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailTemplateType {
    EmailVerification,
    PasswordReset,
    UserInvitation,
    MagicLink,
    AccountLocked,
    MfaEnforcement,
}

impl EmailTemplateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EmailVerification => "email_verification",
            Self::PasswordReset => "password_reset",
            Self::UserInvitation => "user_invitation",
            Self::MagicLink => "magic_link",
            Self::AccountLocked => "account_locked",
            Self::MfaEnforcement => "mfa_enforcement",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "email_verification" => Some(Self::EmailVerification),
            "password_reset" => Some(Self::PasswordReset),
            "user_invitation" => Some(Self::UserInvitation),
            "magic_link" => Some(Self::MagicLink),
            "account_locked" => Some(Self::AccountLocked),
            "mfa_enforcement" => Some(Self::MfaEnforcement),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::EmailVerification,
            Self::PasswordReset,
            Self::UserInvitation,
            Self::MagicLink,
            Self::AccountLocked,
            Self::MfaEnforcement,
        ]
    }

    /// Variables available for this template type.
    pub fn available_variables(&self) -> Vec<&'static str> {
        let mut vars = vec!["org_name", "logo_url", "primary_color"];
        match self {
            Self::EmailVerification => vars.extend_from_slice(&["user_name", "verification_link", "expires_at"]),
            Self::PasswordReset => vars.extend_from_slice(&["user_name", "reset_link", "expires_at", "ip_address"]),
            Self::UserInvitation => vars.extend_from_slice(&["invited_by_name", "tenant_name", "invitation_link", "role_name", "expires_at"]),
            Self::MagicLink => vars.extend_from_slice(&["user_name", "magic_link", "expires_at", "ip_address"]),
            Self::AccountLocked => vars.extend_from_slice(&["user_name", "locked_until", "reason", "failed_attempts"]),
            Self::MfaEnforcement => vars.extend_from_slice(&["user_name", "grace_period_end", "setup_link"]),
        }
        vars
    }

    /// Sample data for preview rendering.
    pub fn sample_variables(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("org_name".into(), "Acme Corp".into());
        vars.insert("logo_url".into(), "".into());
        vars.insert("primary_color".into(), "#2563eb".into());
        match self {
            Self::EmailVerification => {
                vars.insert("user_name".into(), "Jane Doe".into());
                vars.insert("verification_link".into(), "https://app.example.com/verify-email?token=sample-token".into());
                vars.insert("expires_at".into(), "2026-02-08 12:00:00 UTC".into());
            }
            Self::PasswordReset => {
                vars.insert("user_name".into(), "Jane Doe".into());
                vars.insert("reset_link".into(), "https://app.example.com/reset-password?token=sample-token".into());
                vars.insert("expires_at".into(), "2026-02-08 12:00:00 UTC".into());
                vars.insert("ip_address".into(), "192.168.1.1".into());
            }
            Self::UserInvitation => {
                vars.insert("invited_by_name".into(), "John Admin".into());
                vars.insert("tenant_name".into(), "Acme Corp".into());
                vars.insert("invitation_link".into(), "https://app.example.com/invite?token=sample-token".into());
                vars.insert("role_name".into(), "member".into());
                vars.insert("expires_at".into(), "2026-02-14 12:00:00 UTC".into());
            }
            Self::MagicLink => {
                vars.insert("user_name".into(), "Jane Doe".into());
                vars.insert("magic_link".into(), "https://app.example.com/magic?token=sample-token".into());
                vars.insert("expires_at".into(), "2026-02-07 12:15:00 UTC".into());
                vars.insert("ip_address".into(), "192.168.1.1".into());
            }
            Self::AccountLocked => {
                vars.insert("user_name".into(), "Jane Doe".into());
                vars.insert("locked_until".into(), "2026-02-07 13:00:00 UTC".into());
                vars.insert("reason".into(), "too many failed login attempts".into());
                vars.insert("failed_attempts".into(), "5".into());
            }
            Self::MfaEnforcement => {
                vars.insert("user_name".into(), "Jane Doe".into());
                vars.insert("grace_period_end".into(), "2026-02-14 00:00:00 UTC".into());
                vars.insert("setup_link".into(), "https://app.example.com/mfa/setup".into());
            }
        }
        vars
    }
}

/// Custom template from the database.
#[derive(Debug, Clone)]
pub struct CustomEmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

/// Simple {{variable}} substitution.
pub fn render_template(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

/// Default subject lines per template type.
pub fn default_subject(template_type: EmailTemplateType, variables: &HashMap<String, String>) -> String {
    match template_type {
        EmailTemplateType::EmailVerification => "Verify Your Email Address".into(),
        EmailTemplateType::PasswordReset => "Reset Your Password".into(),
        EmailTemplateType::UserInvitation => {
            let tenant = variables.get("tenant_name").cloned().unwrap_or_default();
            format!("You've been invited to join {}", tenant)
        }
        EmailTemplateType::MagicLink => "Your Secure Login Link".into(),
        EmailTemplateType::AccountLocked => "Account Temporarily Locked".into(),
        EmailTemplateType::MfaEnforcement => "Action Required: Enable Multi-Factor Authentication".into(),
    }
}

/// Fetch custom template from DB. Returns None on missing or error (never breaks email sending).
pub async fn fetch_custom_template(
    pool: &sqlx::PgPool,
    tenant_id: uuid::Uuid,
    template_type: EmailTemplateType,
) -> Option<CustomEmailTemplate> {
    let row: Option<(String, String, String)> = sqlx::query_as(
        "SELECT subject, html_body, text_body FROM email_templates WHERE tenant_id = $1 AND template_type = $2 AND is_active = true",
    )
    .bind(tenant_id)
    .bind(template_type.as_str())
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    row.map(|(subject, html_body, text_body)| CustomEmailTemplate {
        subject,
        html_body,
        text_body,
    })
}

/// Render an email using custom template if available, otherwise built-in default.
/// Returns (subject, text_body, html_body).
pub fn render_email(
    template_type: EmailTemplateType,
    custom_template: Option<&CustomEmailTemplate>,
    variables: &HashMap<String, String>,
    branding: &EmailBranding,
) -> (String, String, String) {
    let mut all_vars = variables.clone();
    all_vars.insert("org_name".into(), branding.name.clone());
    all_vars.insert("logo_url".into(), branding.logo_url.clone().unwrap_or_default());
    all_vars.insert("primary_color".into(), branding.primary_color.clone());

    if let Some(custom) = custom_template {
        let subject = render_template(&custom.subject, &all_vars);
        let text = render_template(&custom.text_body, &all_vars);
        let html = render_template(&custom.html_body, &all_vars);
        (subject, text, html)
    } else {
        render_builtin(template_type, &all_vars, branding)
    }
}

/// Render built-in template, returns (subject, text, html).
fn render_builtin(
    template_type: EmailTemplateType,
    variables: &HashMap<String, String>,
    branding: &EmailBranding,
) -> (String, String, String) {
    let subject = default_subject(template_type, variables);
    let empty = String::new();

    let (text, html) = match template_type {
        EmailTemplateType::EmailVerification => {
            let user_name = variables.get("user_name").unwrap_or(&empty);
            let link = variables.get("verification_link").unwrap_or(&empty);
            let expires = variables.get("expires_at").unwrap_or(&empty);
            email_verification_builtin(user_name, link, expires, branding)
        }
        EmailTemplateType::PasswordReset => {
            let user_name = variables.get("user_name").unwrap_or(&empty);
            let link = variables.get("reset_link").unwrap_or(&empty);
            let expires = variables.get("expires_at").unwrap_or(&empty);
            let ip = variables.get("ip_address").unwrap_or(&empty);
            password_reset_builtin(user_name, link, expires, ip, branding)
        }
        EmailTemplateType::UserInvitation => {
            let invited_by = variables.get("invited_by_name").unwrap_or(&empty);
            let tenant = variables.get("tenant_name").unwrap_or(&empty);
            let link = variables.get("invitation_link").unwrap_or(&empty);
            let role = variables.get("role_name");
            let expires = variables.get("expires_at").unwrap_or(&empty);
            user_invitation_builtin(invited_by, tenant, link, role.map(|s| s.as_str()), expires, branding)
        }
        EmailTemplateType::MagicLink => {
            let user_name = variables.get("user_name").unwrap_or(&empty);
            let link = variables.get("magic_link").unwrap_or(&empty);
            let expires = variables.get("expires_at").unwrap_or(&empty);
            let ip = variables.get("ip_address").unwrap_or(&empty);
            magic_link_builtin(user_name, link, expires, ip, branding)
        }
        EmailTemplateType::AccountLocked => {
            let user_name = variables.get("user_name").unwrap_or(&empty);
            let until = variables.get("locked_until").unwrap_or(&empty);
            let reason = variables.get("reason").unwrap_or(&empty);
            let attempts = variables.get("failed_attempts").unwrap_or(&empty);
            account_locked_builtin(user_name, until, reason, attempts, branding)
        }
        EmailTemplateType::MfaEnforcement => {
            let user_name = variables.get("user_name").unwrap_or(&empty);
            let until = variables.get("grace_period_end").unwrap_or(&empty);
            let link = variables.get("setup_link").unwrap_or(&empty);
            mfa_enforcement_builtin(user_name, until, link, branding)
        }
    };

    (subject, text, html)
}

// ============================================================================
// BRANDING
// ============================================================================

/// Branding info passed to all email templates
#[derive(Debug, Clone)]
pub struct EmailBranding {
    /// Organization/app display name (e.g., "Acme Corp"). Falls back to "CoreAuth".
    pub name: String,
    /// Logo URL for the email header. None = no logo.
    pub logo_url: Option<String>,
    /// Primary brand color for buttons/links (e.g., "#e11d48"). Falls back to "#2563eb".
    pub primary_color: String,
}

impl Default for EmailBranding {
    fn default() -> Self {
        Self {
            name: "CoreAuth".to_string(),
            logo_url: None,
            primary_color: "#2563eb".to_string(),
        }
    }
}

impl EmailBranding {
    /// Build from tenant's BrandingSettings, falling back to defaults
    pub fn from_settings(settings: &ciam_models::BrandingSettings) -> Self {
        Self {
            name: settings.app_name.clone().unwrap_or_else(|| "CoreAuth".to_string()),
            logo_url: settings.logo_url.clone(),
            primary_color: settings.primary_color.clone().unwrap_or_else(|| "#2563eb".to_string()),
        }
    }
}

/// Shared HTML header with logo + branding
fn html_header(branding: &EmailBranding) -> String {
    let logo_html = if let Some(url) = &branding.logo_url {
        format!(
            r#"<div style="text-align: center; margin-bottom: 24px;">
                <img src="{}" alt="{}" style="max-height: 48px; max-width: 200px;">
            </div>"#,
            url, branding.name
        )
    } else {
        format!(
            r#"<div style="text-align: center; margin-bottom: 24px;">
                <span style="font-size: 1.5rem; font-weight: 700; color: {};">{}</span>
            </div>"#,
            branding.primary_color, branding.name
        )
    };
    logo_html
}

/// Shared HTML footer
fn html_footer(branding: &EmailBranding) -> String {
    format!(
        r#"<div class="footer">
            <p>Best regards,<br>The {} Team</p>
        </div>"#,
        branding.name
    )
}

/// Text footer
fn text_footer(branding: &EmailBranding) -> String {
    format!("Best regards,\nThe {} Team", branding.name)
}

/// Email verification template
pub fn email_verification(
    user_name: &str,
    verification_link: &str,
    expires_at: &DateTime<Utc>,
    branding: &EmailBranding,
) -> (String, String) {
    let text = format!(
        r#"Hi {},

Please verify your email address by clicking the link below:

{}

This link will expire at {} UTC.

If you didn't create an account, you can safely ignore this email.

{}
"#,
        user_name, verification_link, expires_at, text_footer(branding)
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: {color}; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        {logo}
        <h2>Verify Your Email Address</h2>
        <p>Hi {name},</p>
        <p>Please verify your email address by clicking the button below:</p>
        <a href="{link}" class="button">Verify Email</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{link}</p>
        <p>This link will expire at <strong>{expires} UTC</strong>.</p>
        <p>If you didn't create an account, you can safely ignore this email.</p>
        {footer}
    </div>
</body>
</html>"#,
        color = branding.primary_color,
        logo = html_header(branding),
        name = user_name,
        link = verification_link,
        expires = expires_at,
        footer = html_footer(branding),
    );

    (text, html)
}

/// Password reset template
pub fn password_reset(
    user_name: &str,
    reset_link: &str,
    expires_at: &DateTime<Utc>,
    ip_address: &str,
    branding: &EmailBranding,
) -> (String, String) {
    let text = format!(
        r#"Hi {},

We received a request to reset your password. Click the link below to create a new password:

{}

This link will expire at {} UTC.

This request was made from IP address: {}

If you didn't request a password reset, please ignore this email and your password will remain unchanged.

{}
"#,
        user_name, reset_link, expires_at, ip_address, text_footer(branding)
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: {color}; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .warning {{ background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        {logo}
        <h2>Reset Your Password</h2>
        <p>Hi {name},</p>
        <p>We received a request to reset your password. Click the button below to create a new password:</p>
        <a href="{link}" class="button">Reset Password</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{link}</p>
        <p>This link will expire at <strong>{expires} UTC</strong>.</p>
        <div class="warning">
            <strong>Security Notice:</strong> This request was made from IP address: {ip}
        </div>
        <p>If you didn't request a password reset, please ignore this email and your password will remain unchanged.</p>
        {footer}
    </div>
</body>
</html>"#,
        color = branding.primary_color,
        logo = html_header(branding),
        name = user_name,
        link = reset_link,
        expires = expires_at,
        ip = ip_address,
        footer = html_footer(branding),
    );

    (text, html)
}

/// User invitation template
pub fn user_invitation(
    invited_by_name: &str,
    tenant_name: &str,
    invitation_link: &str,
    role_name: Option<&str>,
    expires_at: &DateTime<Utc>,
    branding: &EmailBranding,
) -> (String, String) {
    let role_text = role_name
        .map(|r| format!(" as a {}", r))
        .unwrap_or_default();

    let text = format!(
        r#"Hello,

{} has invited you to join {}{}.

Click the link below to accept the invitation and create your account:

{}

This invitation will expire at {} UTC.

If you don't want to accept this invitation, you can safely ignore this email.

{}
"#,
        invited_by_name, tenant_name, role_text, invitation_link, expires_at, text_footer(branding)
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: {color}; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .info-box {{ background-color: #e7f3ff; border-left: 4px solid {color}; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        {logo}
        <h2>You've Been Invited!</h2>
        <p>Hello,</p>
        <p><strong>{invited_by}</strong> has invited you to join <strong>{tenant}</strong>{role}.</p>
        <a href="{link}" class="button">Accept Invitation</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{link}</p>
        <div class="info-box">
            <p><strong>Important:</strong> This invitation will expire at <strong>{expires} UTC</strong>.</p>
        </div>
        <p>If you don't want to accept this invitation, you can safely ignore this email.</p>
        {footer}
    </div>
</body>
</html>"#,
        color = branding.primary_color,
        logo = html_header(branding),
        invited_by = invited_by_name,
        tenant = tenant_name,
        role = role_text,
        link = invitation_link,
        expires = expires_at,
        footer = html_footer(branding),
    );

    (text, html)
}

/// Magic link template
pub fn magic_link(
    user_name: &str,
    magic_link: &str,
    expires_at: &DateTime<Utc>,
    ip_address: &str,
    branding: &EmailBranding,
) -> (String, String) {
    let text = format!(
        r#"Hi {},

Here's your secure login link:

{}

This link will expire at {} UTC.

This request was made from IP address: {}

If you didn't request this login link, please ignore this email.

{}
"#,
        user_name, magic_link, expires_at, ip_address, text_footer(branding)
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: {color}; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .warning {{ background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        {logo}
        <h2>Your Secure Login Link</h2>
        <p>Hi {name},</p>
        <p>Click the button below to securely log in to your account:</p>
        <a href="{link}" class="button">Log In</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{link}</p>
        <p>This link will expire at <strong>{expires} UTC</strong>.</p>
        <div class="warning">
            <strong>Security Notice:</strong> This request was made from IP address: {ip}
        </div>
        <p>If you didn't request this login link, please ignore this email.</p>
        {footer}
    </div>
</body>
</html>"#,
        color = branding.primary_color,
        logo = html_header(branding),
        name = user_name,
        link = magic_link,
        expires = expires_at,
        ip = ip_address,
        footer = html_footer(branding),
    );

    (text, html)
}

/// Account locked notification
pub fn account_locked(
    user_name: &str,
    locked_until: &DateTime<Utc>,
    reason: &str,
    failed_attempts: i64,
    branding: &EmailBranding,
) -> (String, String) {
    let text = format!(
        r#"Hi {},

Your account has been temporarily locked due to {}.

Details:
- Failed login attempts: {}
- Locked until: {} UTC

Your account will be automatically unlocked after this time. If you believe this was a mistake or need immediate access, please contact support.

{}
"#,
        user_name, reason, failed_attempts, locked_until, text_footer(branding)
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .alert {{ background-color: #f8d7da; border-left: 4px solid #dc3545; padding: 12px; margin: 20px 0; }}
        .info {{ background-color: #e7f3ff; border-left: 4px solid {color}; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        {logo}
        <h2>Account Temporarily Locked</h2>
        <p>Hi {name},</p>
        <div class="alert">
            <strong>Security Alert:</strong> Your account has been temporarily locked due to {reason}.
        </div>
        <div class="info">
            <p><strong>Details:</strong></p>
            <ul>
                <li>Failed login attempts: {attempts}</li>
                <li>Locked until: <strong>{until} UTC</strong></li>
            </ul>
        </div>
        <p>Your account will be automatically unlocked after this time.</p>
        <p>If you believe this was a mistake or need immediate access, please contact support.</p>
        {footer}
    </div>
</body>
</html>"#,
        color = branding.primary_color,
        logo = html_header(branding),
        name = user_name,
        reason = reason,
        attempts = failed_attempts,
        until = locked_until,
        footer = html_footer(branding),
    );

    (text, html)
}

// ============================================================================
// BUILTIN TEMPLATE WRAPPERS (for render_builtin — accept &str params)
// ============================================================================

fn email_verification_builtin(user_name: &str, verification_link: &str, expires_at: &str, branding: &EmailBranding) -> (String, String) {
    let text = format!(
        "Hi {},\n\nPlease verify your email address by clicking the link below:\n\n{}\n\nThis link will expire at {}.\n\nIf you didn't create an account, you can safely ignore this email.\n\n{}",
        user_name, verification_link, expires_at, text_footer(branding)
    );
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{{font-family:Arial,sans-serif;line-height:1.6;color:#333}}.container{{max-width:600px;margin:0 auto;padding:20px}}.button{{display:inline-block;padding:12px 24px;background-color:{color};color:white;text-decoration:none;border-radius:4px;margin:20px 0}}.footer{{margin-top:30px;padding-top:20px;border-top:1px solid #ddd;font-size:12px;color:#666}}</style></head><body><div class="container">{logo}<h2>Verify Your Email Address</h2><p>Hi {name},</p><p>Please verify your email address by clicking the button below:</p><a href="{link}" class="button">Verify Email</a><p>Or copy and paste this link into your browser:</p><p style="word-break:break-all;color:#666;">{link}</p><p>This link will expire at <strong>{expires}</strong>.</p><p>If you didn't create an account, you can safely ignore this email.</p>{footer}</div></body></html>"#,
        color = branding.primary_color, logo = html_header(branding), name = user_name, link = verification_link, expires = expires_at, footer = html_footer(branding),
    );
    (text, html)
}

fn password_reset_builtin(user_name: &str, reset_link: &str, expires_at: &str, ip_address: &str, branding: &EmailBranding) -> (String, String) {
    let text = format!(
        "Hi {},\n\nWe received a request to reset your password. Click the link below to create a new password:\n\n{}\n\nThis link will expire at {}.\n\nThis request was made from IP address: {}\n\nIf you didn't request a password reset, please ignore this email.\n\n{}",
        user_name, reset_link, expires_at, ip_address, text_footer(branding)
    );
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{{font-family:Arial,sans-serif;line-height:1.6;color:#333}}.container{{max-width:600px;margin:0 auto;padding:20px}}.button{{display:inline-block;padding:12px 24px;background-color:{color};color:white;text-decoration:none;border-radius:4px;margin:20px 0}}.warning{{background-color:#fff3cd;border-left:4px solid #ffc107;padding:12px;margin:20px 0}}.footer{{margin-top:30px;padding-top:20px;border-top:1px solid #ddd;font-size:12px;color:#666}}</style></head><body><div class="container">{logo}<h2>Reset Your Password</h2><p>Hi {name},</p><p>We received a request to reset your password. Click the button below:</p><a href="{link}" class="button">Reset Password</a><p>Or copy and paste this link:</p><p style="word-break:break-all;color:#666;">{link}</p><p>This link will expire at <strong>{expires}</strong>.</p><div class="warning"><strong>Security Notice:</strong> This request was made from IP address: {ip}</div><p>If you didn't request this, please ignore this email.</p>{footer}</div></body></html>"#,
        color = branding.primary_color, logo = html_header(branding), name = user_name, link = reset_link, expires = expires_at, ip = ip_address, footer = html_footer(branding),
    );
    (text, html)
}

fn user_invitation_builtin(invited_by: &str, tenant_name: &str, invitation_link: &str, role_name: Option<&str>, expires_at: &str, branding: &EmailBranding) -> (String, String) {
    let role_text = role_name.map(|r| format!(" as a {}", r)).unwrap_or_default();
    let text = format!(
        "Hello,\n\n{} has invited you to join {}{}.\n\nClick the link below to accept:\n\n{}\n\nThis invitation will expire at {}.\n\nIf you don't want to accept, you can safely ignore this email.\n\n{}",
        invited_by, tenant_name, role_text, invitation_link, expires_at, text_footer(branding)
    );
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{{font-family:Arial,sans-serif;line-height:1.6;color:#333}}.container{{max-width:600px;margin:0 auto;padding:20px}}.button{{display:inline-block;padding:12px 24px;background-color:{color};color:white;text-decoration:none;border-radius:4px;margin:20px 0}}.info-box{{background-color:#e7f3ff;border-left:4px solid {color};padding:12px;margin:20px 0}}.footer{{margin-top:30px;padding-top:20px;border-top:1px solid #ddd;font-size:12px;color:#666}}</style></head><body><div class="container">{logo}<h2>You've Been Invited!</h2><p>Hello,</p><p><strong>{invited_by}</strong> has invited you to join <strong>{tenant}</strong>{role}.</p><a href="{link}" class="button">Accept Invitation</a><p>Or copy and paste this link:</p><p style="word-break:break-all;color:#666;">{link}</p><div class="info-box"><p><strong>Important:</strong> This invitation expires at <strong>{expires}</strong>.</p></div>{footer}</div></body></html>"#,
        color = branding.primary_color, logo = html_header(branding), invited_by = invited_by, tenant = tenant_name, role = role_text, link = invitation_link, expires = expires_at, footer = html_footer(branding),
    );
    (text, html)
}

fn magic_link_builtin(user_name: &str, link_url: &str, expires_at: &str, ip_address: &str, branding: &EmailBranding) -> (String, String) {
    let text = format!(
        "Hi {},\n\nHere's your secure login link:\n\n{}\n\nThis link will expire at {}.\n\nThis request was made from IP address: {}\n\nIf you didn't request this, please ignore this email.\n\n{}",
        user_name, link_url, expires_at, ip_address, text_footer(branding)
    );
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{{font-family:Arial,sans-serif;line-height:1.6;color:#333}}.container{{max-width:600px;margin:0 auto;padding:20px}}.button{{display:inline-block;padding:12px 24px;background-color:{color};color:white;text-decoration:none;border-radius:4px;margin:20px 0}}.warning{{background-color:#fff3cd;border-left:4px solid #ffc107;padding:12px;margin:20px 0}}.footer{{margin-top:30px;padding-top:20px;border-top:1px solid #ddd;font-size:12px;color:#666}}</style></head><body><div class="container">{logo}<h2>Your Secure Login Link</h2><p>Hi {name},</p><p>Click the button below to securely log in:</p><a href="{link}" class="button">Log In</a><p>Or copy and paste this link:</p><p style="word-break:break-all;color:#666;">{link}</p><p>This link will expire at <strong>{expires}</strong>.</p><div class="warning"><strong>Security Notice:</strong> Request from IP: {ip}</div>{footer}</div></body></html>"#,
        color = branding.primary_color, logo = html_header(branding), name = user_name, link = link_url, expires = expires_at, ip = ip_address, footer = html_footer(branding),
    );
    (text, html)
}

fn account_locked_builtin(user_name: &str, locked_until: &str, reason: &str, failed_attempts: &str, branding: &EmailBranding) -> (String, String) {
    let text = format!(
        "Hi {},\n\nYour account has been temporarily locked due to {}.\n\nDetails:\n- Failed login attempts: {}\n- Locked until: {}\n\nYour account will be automatically unlocked after this time.\n\n{}",
        user_name, reason, failed_attempts, locked_until, text_footer(branding)
    );
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{{font-family:Arial,sans-serif;line-height:1.6;color:#333}}.container{{max-width:600px;margin:0 auto;padding:20px}}.alert{{background-color:#f8d7da;border-left:4px solid #dc3545;padding:12px;margin:20px 0}}.info{{background-color:#e7f3ff;border-left:4px solid {color};padding:12px;margin:20px 0}}.footer{{margin-top:30px;padding-top:20px;border-top:1px solid #ddd;font-size:12px;color:#666}}</style></head><body><div class="container">{logo}<h2>Account Temporarily Locked</h2><p>Hi {name},</p><div class="alert"><strong>Security Alert:</strong> Your account has been temporarily locked due to {reason}.</div><div class="info"><p><strong>Details:</strong></p><ul><li>Failed login attempts: {attempts}</li><li>Locked until: <strong>{until}</strong></li></ul></div><p>Your account will be automatically unlocked after this time.</p>{footer}</div></body></html>"#,
        color = branding.primary_color, logo = html_header(branding), name = user_name, reason = reason, attempts = failed_attempts, until = locked_until, footer = html_footer(branding),
    );
    (text, html)
}

fn mfa_enforcement_builtin(user_name: &str, grace_period_end: &str, setup_link: &str, branding: &EmailBranding) -> (String, String) {
    let text = format!(
        "Hi {},\n\nYour organization now requires multi-factor authentication (MFA) for all accounts.\n\nYou have until {} to set up MFA.\n\nSet up MFA now: {}\n\nAfter the grace period, you will need MFA to log in.\n\n{}",
        user_name, grace_period_end, setup_link, text_footer(branding)
    );
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{{font-family:Arial,sans-serif;line-height:1.6;color:#333}}.container{{max-width:600px;margin:0 auto;padding:20px}}.button{{display:inline-block;padding:12px 24px;background-color:{color};color:white;text-decoration:none;border-radius:4px;margin:20px 0}}.warning{{background-color:#fff3cd;border-left:4px solid #ffc107;padding:12px;margin:20px 0}}.footer{{margin-top:30px;padding-top:20px;border-top:1px solid #ddd;font-size:12px;color:#666}}</style></head><body><div class="container">{logo}<h2>Action Required: Enable MFA</h2><p>Hi {name},</p><div class="warning"><strong>Important:</strong> Your organization now requires MFA for all accounts.</div><p>You have until <strong>{until}</strong> to set up MFA.</p><a href="{link}" class="button">Set Up MFA Now</a><p>After the grace period, MFA will be required to log in.</p>{footer}</div></body></html>"#,
        color = branding.primary_color, logo = html_header(branding), name = user_name, until = grace_period_end, link = setup_link, footer = html_footer(branding),
    );
    (text, html)
}

// ============================================================================
// LEGACY PUBLIC FUNCTIONS (kept for backward compatibility until all callers migrate)
// ============================================================================

/// MFA enforcement notification
pub fn mfa_enforcement_notification(
    user_name: &str,
    grace_period_end: &DateTime<Utc>,
    setup_link: &str,
    branding: &EmailBranding,
) -> (String, String) {
    let text = format!(
        r#"Hi {},

Your organization now requires multi-factor authentication (MFA) for all accounts.

You have until {} UTC to set up MFA on your account.

Set up MFA now:
{}

After the grace period ends, you will be required to enable MFA before you can log in.

Why MFA?
MFA adds an extra layer of security to your account by requiring a second form of verification in addition to your password.

{}
"#,
        user_name, grace_period_end, setup_link, text_footer(branding)
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: {color}; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .warning {{ background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 12px; margin: 20px 0; }}
        .info {{ background-color: #e7f3ff; border-left: 4px solid {color}; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        {logo}
        <h2>Action Required: Enable Multi-Factor Authentication</h2>
        <p>Hi {name},</p>
        <div class="warning">
            <strong>Important:</strong> Your organization now requires multi-factor authentication (MFA) for all accounts.
        </div>
        <p>You have until <strong>{until} UTC</strong> to set up MFA on your account.</p>
        <a href="{link}" class="button">Set Up MFA Now</a>
        <div class="info">
            <p><strong>What happens next?</strong></p>
            <p>After the grace period ends, you will be required to enable MFA before you can log in to your account.</p>
        </div>
        <p><strong>Why MFA?</strong></p>
        <p>MFA adds an extra layer of security to your account by requiring a second form of verification in addition to your password. This helps protect your account even if your password is compromised.</p>
        {footer}
    </div>
</body>
</html>"#,
        color = branding.primary_color,
        logo = html_header(branding),
        name = user_name,
        until = grace_period_end,
        link = setup_link,
        footer = html_footer(branding),
    );

    (text, html)
}
