use chrono::{DateTime, Utc};

/// Email verification template
pub fn email_verification(
    user_name: &str,
    verification_link: &str,
    expires_at: &DateTime<Utc>,
) -> (String, String) {
    let text = format!(
        r#"Hi {},

Please verify your email address by clicking the link below:

{}

This link will expire at {} UTC.

If you didn't create an account, you can safely ignore this email.

Best regards,
The CIAM Team
"#,
        user_name, verification_link, expires_at
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: #007bff; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Verify Your Email Address</h2>
        <p>Hi {},</p>
        <p>Please verify your email address by clicking the button below:</p>
        <a href="{}" class="button">Verify Email</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{}</p>
        <p>This link will expire at <strong>{} UTC</strong>.</p>
        <p>If you didn't create an account, you can safely ignore this email.</p>
        <div class="footer">
            <p>Best regards,<br>The CIAM Team</p>
        </div>
    </div>
</body>
</html>"#,
        user_name, verification_link, verification_link, expires_at
    );

    (text, html)
}

/// Password reset template
pub fn password_reset(
    user_name: &str,
    reset_link: &str,
    expires_at: &DateTime<Utc>,
    ip_address: &str,
) -> (String, String) {
    let text = format!(
        r#"Hi {},

We received a request to reset your password. Click the link below to create a new password:

{}

This link will expire at {} UTC.

This request was made from IP address: {}

If you didn't request a password reset, please ignore this email and your password will remain unchanged.

Best regards,
The CIAM Team
"#,
        user_name, reset_link, expires_at, ip_address
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: #007bff; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .warning {{ background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Reset Your Password</h2>
        <p>Hi {},</p>
        <p>We received a request to reset your password. Click the button below to create a new password:</p>
        <a href="{}" class="button">Reset Password</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{}</p>
        <p>This link will expire at <strong>{} UTC</strong>.</p>
        <div class="warning">
            <strong>Security Notice:</strong> This request was made from IP address: {}
        </div>
        <p>If you didn't request a password reset, please ignore this email and your password will remain unchanged.</p>
        <div class="footer">
            <p>Best regards,<br>The CIAM Team</p>
        </div>
    </div>
</body>
</html>"#,
        user_name, reset_link, reset_link, expires_at, ip_address
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
) -> (String, String) {
    let role_text = role_name
        .map(|r| format!(" as a {}", r))
        .unwrap_or_default();

    let text = format!(
        r#"Hello,

{} has invited you to join {} on CIAM{}.

Click the link below to accept the invitation and create your account:

{}

This invitation will expire at {} UTC.

If you don't want to accept this invitation, you can safely ignore this email.

Best regards,
The CIAM Team
"#,
        invited_by_name, tenant_name, role_text, invitation_link, expires_at
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: #28a745; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .info-box {{ background-color: #e7f3ff; border-left: 4px solid #007bff; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>You've Been Invited!</h2>
        <p>Hello,</p>
        <p><strong>{}</strong> has invited you to join <strong>{}</strong> on CIAM{}.</p>
        <a href="{}" class="button">Accept Invitation</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{}</p>
        <div class="info-box">
            <p><strong>Important:</strong> This invitation will expire at <strong>{} UTC</strong>.</p>
        </div>
        <p>If you don't want to accept this invitation, you can safely ignore this email.</p>
        <div class="footer">
            <p>Best regards,<br>The CIAM Team</p>
        </div>
    </div>
</body>
</html>"#,
        invited_by_name, tenant_name, role_text, invitation_link, invitation_link, expires_at
    );

    (text, html)
}

/// Magic link template
pub fn magic_link(
    user_name: &str,
    magic_link: &str,
    expires_at: &DateTime<Utc>,
    ip_address: &str,
) -> (String, String) {
    let text = format!(
        r#"Hi {},

Here's your secure login link:

{}

This link will expire at {} UTC.

This request was made from IP address: {}

If you didn't request this login link, please ignore this email.

Best regards,
The CIAM Team
"#,
        user_name, magic_link, expires_at, ip_address
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: #6f42c1; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .warning {{ background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Your Secure Login Link</h2>
        <p>Hi {},</p>
        <p>Click the button below to securely log in to your account:</p>
        <a href="{}" class="button">Log In</a>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #666;">{}</p>
        <p>This link will expire at <strong>{} UTC</strong>.</p>
        <div class="warning">
            <strong>Security Notice:</strong> This request was made from IP address: {}
        </div>
        <p>If you didn't request this login link, please ignore this email.</p>
        <div class="footer">
            <p>Best regards,<br>The CIAM Team</p>
        </div>
    </div>
</body>
</html>"#,
        user_name, magic_link, magic_link, expires_at, ip_address
    );

    (text, html)
}

/// Account locked notification
pub fn account_locked(
    user_name: &str,
    locked_until: &DateTime<Utc>,
    reason: &str,
    failed_attempts: i64,
) -> (String, String) {
    let text = format!(
        r#"Hi {},

Your account has been temporarily locked due to {}.

Details:
- Failed login attempts: {}
- Locked until: {} UTC

Your account will be automatically unlocked after this time. If you believe this was a mistake or need immediate access, please contact support.

Best regards,
The CIAM Team
"#,
        user_name, reason, failed_attempts, locked_until
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
        .info {{ background-color: #e7f3ff; border-left: 4px solid #007bff; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Account Temporarily Locked</h2>
        <p>Hi {},</p>
        <div class="alert">
            <strong>Security Alert:</strong> Your account has been temporarily locked due to {}.
        </div>
        <div class="info">
            <p><strong>Details:</strong></p>
            <ul>
                <li>Failed login attempts: {}</li>
                <li>Locked until: <strong>{} UTC</strong></li>
            </ul>
        </div>
        <p>Your account will be automatically unlocked after this time.</p>
        <p>If you believe this was a mistake or need immediate access, please contact support.</p>
        <div class="footer">
            <p>Best regards,<br>The CIAM Team</p>
        </div>
    </div>
</body>
</html>"#,
        user_name, reason, failed_attempts, locked_until
    );

    (text, html)
}

/// MFA enforcement notification
pub fn mfa_enforcement_notification(
    user_name: &str,
    grace_period_end: &DateTime<Utc>,
    setup_link: &str,
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

Best regards,
The CIAM Team
"#,
        user_name, grace_period_end, setup_link
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: #007bff; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .warning {{ background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 12px; margin: 20px 0; }}
        .info {{ background-color: #e7f3ff; border-left: 4px solid #007bff; padding: 12px; margin: 20px 0; }}
        .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Action Required: Enable Multi-Factor Authentication</h2>
        <p>Hi {},</p>
        <div class="warning">
            <strong>Important:</strong> Your organization now requires multi-factor authentication (MFA) for all accounts.
        </div>
        <p>You have until <strong>{} UTC</strong> to set up MFA on your account.</p>
        <a href="{}" class="button">Set Up MFA Now</a>
        <div class="info">
            <p><strong>What happens next?</strong></p>
            <p>After the grace period ends, you will be required to enable MFA before you can log in to your account.</p>
        </div>
        <p><strong>Why MFA?</strong></p>
        <p>MFA adds an extra layer of security to your account by requiring a second form of verification in addition to your password. This helps protect your account even if your password is compromised.</p>
        <div class="footer">
            <p>Best regards,<br>The CIAM Team</p>
        </div>
    </div>
</body>
</html>"#,
        user_name, grace_period_end, setup_link
    );

    (text, html)
}
