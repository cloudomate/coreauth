# Email & SMS Configuration

## Overview

CoreAuth sends emails for verification, password resets, invitations, MFA notices, and magic links. SMS is used for phone-based MFA codes.

## Email Configuration

### Development (MailHog)

MailHog captures all outgoing emails locally for testing.

Add to `.env`:

```env
EMAIL_PROVIDER=mailhog
EMAIL_FROM=noreply@coreauth.dev
EMAIL_FROM_NAME=CoreAuth
MAILHOG_HOST=localhost
MAILHOG_PORT=1025
```

All emails are captured and viewable in the MailHog web UI. No emails are sent externally.

### Production (SMTP)

For production, use a real email provider (SendGrid, AWS SES, Gmail, etc.):

```env
EMAIL_PROVIDER=smtp
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
EMAIL_FROM=noreply@yourdomain.com
EMAIL_FROM_NAME=YourApp
```

### Email Types

CoreAuth sends emails for:

- Email verification (signup)
- Password reset
- Invitation to organization
- MFA enforcement notices
- Magic link authentication
- Security alerts

### Branded Emails

Email templates automatically use tenant branding (logo, colors, app name) configured via the branding API:

```bash
curl -X PUT "http://localhost:8000/api/organizations/${TENANT_ID}/branding" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "app_name": "Acme Corp",
    "logo_url": "https://acme.com/logo.png",
    "primary_color": "#e11d48"
  }'
```

### Custom Email Templates

Manage email templates per-tenant via the API:

```bash
# List templates
curl "http://localhost:8000/api/tenants/${TENANT_ID}/email-templates" \
  -H "Authorization: Bearer ${TOKEN}"

# Update a template
curl -X PUT "http://localhost:8000/api/tenants/${TENANT_ID}/email-templates/verification" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "subject": "Verify your email for {{app_name}}",
    "html_body": "<p>Click <a href=\"{{verification_link}}\">here</a> to verify.</p>"
  }'
```

---

## SMS Configuration

### Development (SMPP Gateway)

```env
SMS_ENABLED=true
SMS_PROVIDER=smpp
SMPP_HOST=localhost
SMPP_PORT=2775
SMPP_SYSTEM_ID=test_client
SMPP_PASSWORD=password
SMPP_SOURCE_ADDR=CoreAuth
```

### Production (Twilio)

```env
SMS_ENABLED=true
SMS_PROVIDER=twilio
TWILIO_ACCOUNT_SID=your-account-sid
TWILIO_AUTH_TOKEN=your-auth-token
TWILIO_FROM_NUMBER=+1234567890
```

### SMS Use Cases

- Phone verification
- SMS-based MFA codes
- Security alerts

---

## Implementation

### Email Service

Located in `coreauth-core/crates/auth/src/email/`:

- `mod.rs` - Email provider abstraction
- `service.rs` - Email sending logic
- `templates.rs` - Template rendering

### SMS Service

Located in `coreauth-core/crates/auth/src/sms/`:

- `mod.rs` - SMS provider abstraction
- `service.rs` - SMS sending logic

---

## Troubleshooting

### Emails not being sent

- Check `EMAIL_PROVIDER` is set correctly in `.env`
- Verify SMTP/MailHog host is reachable
- Check application logs: `docker compose logs -f coreauth-core`

### Verification links not working

- Verify `APP_URL` in `.env` matches your deployment URL
- Check token expiration settings

### SMS not being sent

- Verify `SMS_ENABLED=true` in `.env`
- Check provider credentials
- Verify phone numbers use E.164 format (`+1234567890`)

---

## Security Considerations

- Never commit `.env` files to git
- Use TLS for SMTP in production
- Rate limit SMS sends to prevent abuse
- Validate email addresses before sending
- Set appropriate token expiration times
