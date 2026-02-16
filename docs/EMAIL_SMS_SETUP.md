# Email & SMS Configuration

## Overview

CoreAuth supports email verification, password resets, and SMS-based 2FA. This guide shows how to configure Mailhog (email) and SMPP Gateway (SMS) for testing.

## Email Configuration (Mailhog)

Mailhog is a testing email server that captures all outgoing emails for development/testing.

### Configuration

Add to `.env`:

```env
SMTP_HOST=datacore.lan
SMTP_PORT=1025
SMTP_USERNAME=
SMTP_PASSWORD=
SMTP_FROM_EMAIL=noreply@coreauth.dev
SMTP_FROM_NAME=CoreAuth
```

### Access

- **Web UI**: https://mailhog.imys.in
- **SMTP Server**: datacore.lan:1025

### Features

- All emails sent by CoreAuth are captured
- View email content (HTML & plain text)
- Test email templates
- No actual emails are sent externally

### Email Templates

CoreAuth sends emails for:
- Email verification (signup)
- Password reset
- Invitation to organization
- MFA setup
- Security alerts

## SMS Configuration (SMPP Gateway)

SMPP (Short Message Peer-to-Peer) gateway for testing SMS-based 2FA.

### Configuration

Add to `.env`:

```env
SMS_ENABLED=true
SMS_PROVIDER=smpp
SMPP_HOST=datacore.lan
SMPP_PORT=2775
SMPP_SYSTEM_ID=test_client
SMPP_PASSWORD=password
SMPP_SYSTEM_TYPE=
SMPP_SOURCE_ADDR=CoreAuth
```

### Connection Details

- **Host**: datacore.lan (or sms.imys.in)
- **Port**: 2775
- **System ID**: Any value accepted (use `test_client`)
- **Password**: Any value accepted (use `password`)

### SMS Use Cases

CoreAuth sends SMS for:
- Phone verification
- SMS-based 2FA codes
- Password reset via SMS
- Security alerts to phone

### Testing SMS

1. Enable SMS in tenant settings
2. Add phone number to user profile
3. Request 2FA code
4. Code is sent via SMPP gateway
5. View captured SMS in gateway logs

## Implementation in Code

### Email Service

Located in `coreauth-core/crates/auth/src/email.rs`:

```rust
pub async fn send_verification_email(
    &self,
    to: &str,
    verification_token: &str,
) -> Result<()> {
    let verification_link = format!(
        "{}/verify?token={}",
        self.base_url, verification_token
    );

    let message = EmailMessage {
        to: vec![to.to_string()],
        subject: "Verify your email".to_string(),
        html_body: format!(
            "<p>Click <a href='{}'>here</a> to verify your email.</p>",
            verification_link
        ),
        text_body: format!("Verify your email: {}", verification_link),
    };

    self.send(message).await
}
```

### SMS Service

Located in `coreauth-core/crates/auth/src/sms.rs`:

```rust
pub async fn send_verification_code(
    &self,
    phone: &str,
    code: &str,
) -> Result<()> {
    let message = SmsMessage {
        to: phone.to_string(),
        body: format!("Your CoreAuth verification code is: {}", code),
    };

    self.send(message).await
}
```

## Social Login (Future)

### Supported Providers

CoreAuth will support tenant signup via:
- Google OAuth
- GitHub OAuth
- Microsoft OAuth
- LinkedIn OAuth

### Configuration

Add to `.env`:

```env
# Google
GOOGLE_CLIENT_ID=your-client-id
GOOGLE_CLIENT_SECRET=your-client-secret
GOOGLE_REDIRECT_URI=http://localhost:8000/api/auth/callback/google

# GitHub
GITHUB_CLIENT_ID=your-client-id
GITHUB_CLIENT_SECRET=your-client-secret
GITHUB_REDIRECT_URI=http://localhost:8000/api/auth/callback/github
```

### Implementation Plan

1. Add OAuth provider configuration UI
2. Implement OAuth callback handlers
3. Link social accounts to existing tenants
4. Allow tenant creation via social signup
5. Support multiple OAuth providers per tenant

### OAuth Flow

```
User → "Sign up with Google"
     → Redirect to Google OAuth
     → User authorizes
     → Redirect back with auth code
     → Exchange code for tokens
     → Get user profile
     → Create tenant + admin user
     → Auto-login
```

## Testing Checklist

### Email Testing

- [ ] Send verification email
- [ ] Receive email in Mailhog
- [ ] Click verification link
- [ ] Test password reset email
- [ ] Test invitation email
- [ ] Verify email templates render correctly

### SMS Testing

- [ ] Enable SMS 2FA
- [ ] Request SMS code
- [ ] Receive code via SMPP
- [ ] Verify code works
- [ ] Test rate limiting
- [ ] Test invalid phone numbers

### Social Login Testing (Future)

- [ ] Configure OAuth providers
- [ ] Test signup with Google
- [ ] Test signup with GitHub
- [ ] Test account linking
- [ ] Test error handling
- [ ] Test token refresh

## Troubleshooting

### Email Issues

**Problem**: Emails not being sent
- Check SMTP host and port
- Verify Mailhog is running
- Check application logs for errors

**Problem**: Verification links don't work
- Verify `APP_URL` in `.env`
- Check token expiration settings
- Verify database connection

### SMS Issues

**Problem**: SMS not being sent
- Check SMPP connection details
- Verify gateway is accessible
- Check phone number format (E.164)

**Problem**: SMS codes not working
- Verify TOTP settings match
- Check code expiration (usually 5 minutes)
- Verify time sync on server

## Production Configuration

### Email (SendGrid/AWS SES)

For production, replace Mailhog with a real email provider:

```env
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
SMTP_FROM_EMAIL=noreply@yourdomain.com
SMTP_FROM_NAME=YourApp
```

### SMS (Twilio/AWS SNS)

For production, use a real SMS provider:

```env
SMS_PROVIDER=twilio
TWILIO_ACCOUNT_SID=your-account-sid
TWILIO_AUTH_TOKEN=your-auth-token
TWILIO_PHONE_NUMBER=+1234567890
```

## Security Considerations

- Never commit `.env` files to git
- Use strong passwords in production
- Enable TLS for SMTP in production
- Rate limit SMS sends to prevent abuse
- Validate email addresses before sending
- Sanitize user input in email templates
- Use signed tokens for verification links
- Set appropriate token expiration times

## References

- [Mailhog Documentation](https://github.com/mailhog/MailHog)
- [SMPP Protocol Specification](https://smpp.org/)
- [OAuth 2.0 RFC](https://tools.ietf.org/html/rfc6749)
- [SendGrid Documentation](https://docs.sendgrid.com/)
- [Twilio SMS API](https://www.twilio.com/docs/sms)
