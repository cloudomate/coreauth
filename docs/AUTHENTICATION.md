# Authentication

## Overview

CIAM supports multiple authentication methods to accommodate different use cases and security requirements.

## Authentication Methods

### 1. Password-Based Authentication

#### Registration

**Endpoint:** `POST /api/auth/register`

**Request:**
```json
{
  "tenant_id": "acme-corp",
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "phone": "+1234567890"
}
```

**Response:**
```json
{
  "Success": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "tenant_id": "uuid",
      "is_verified": false
    },
    "access_token": "eyJ...",
    "refresh_token": "eyJ...",
    "expires_in": 3600
  }
}
```

**Features:**
- Argon2id password hashing
- Email verification required
- Automatic verification email sent
- Rate limited (5 requests/minute per IP)

#### Login

**Endpoint:** `POST /api/auth/login`

**Request:**
```json
{
  "tenant_id": "acme-corp",
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Response:**
```json
{
  "Success": {
    "user": { ... },
    "access_token": "eyJ...",
    "refresh_token": "eyJ...",
    "expires_in": 3600
  }
}
```

**Features:**
- Failed attempt tracking
- Account lockout after 5 failures (15 min)
- MFA challenge if enabled
- Rate limited (10 requests/minute per IP)

### 2. OpenID Connect (OIDC)

#### Supported Providers

##### Auth0
- Full OIDC support
- Group synchronization via custom claim
- Automatic user provisioning

##### Google Workspace
- Google OAuth 2.0
- Group membership sync
- Email-based user matching

##### Microsoft Entra ID (Azure AD)
- Azure AD integration
- Group claim support
- Enterprise SSO

#### OIDC Login Flow

```
1. User clicks "Login with Provider"
   ↓
2. Frontend: GET /api/oidc/login?provider_id={id}&tenant_id={tenant}
   ↓
3. Backend generates OAuth state
   ↓
4. Redirect to OIDC provider
   ↓
5. User authenticates at provider
   ↓
6. Provider redirects to: /api/oidc/callback?code=...&state=...
   ↓
7. Backend validates state & exchanges code for tokens
   ↓
8. Extract user info and groups from JWT
   ↓
9. Create/update user account
   ↓
10. Sync groups to roles
   ↓
11. Return CIAM access & refresh tokens
```

#### OIDC Provider Configuration

**Endpoint:** `POST /api/oidc/providers`

**Request:**
```json
{
  "tenant_id": "uuid",
  "name": "Auth0",
  "provider_type": "auth0",
  "client_id": "your-client-id",
  "client_secret": "your-client-secret",
  "authorization_endpoint": "https://tenant.auth0.com/authorize",
  "token_endpoint": "https://tenant.auth0.com/oauth/token",
  "userinfo_endpoint": "https://tenant.auth0.com/userinfo",
  "jwks_uri": "https://tenant.auth0.com/.well-known/jwks.json",
  "scopes": ["openid", "profile", "email"],
  "groups_claim": "https://schemas.auth0.com/groups",
  "group_role_mappings": {
    "admin-group": "admin",
    "user-group": "user"
  }
}
```

#### Group Synchronization

Groups from OIDC providers are automatically mapped to tenant roles:

1. Extract groups from JWT claim (configurable path)
2. Match group names to role names via mappings
3. Assign roles to user
4. Remove roles not in current groups

**Example:**
```json
{
  "group_role_mappings": {
    "Engineering": "developer",
    "Product": "product_manager",
    "Admins": "admin"
  }
}
```

### 3. Multi-Factor Authentication (MFA)

#### TOTP Enrollment

**Endpoint:** `POST /api/mfa/enroll/totp`

**Request:**
```json
{
  "name": "My Authenticator"
}
```

**Response:**
```json
{
  "method_id": "uuid",
  "secret": "BASE32SECRET",
  "qr_code": "data:image/png;base64,...",
  "backup_codes": [
    "12345678",
    "87654321",
    ...
  ]
}
```

**Flow:**
1. User requests TOTP enrollment
2. Backend generates secret
3. QR code created for secret
4. User scans QR code with authenticator app
5. User verifies with first code
6. Method activated, backup codes provided

#### TOTP Verification

**Endpoint:** `POST /api/mfa/totp/{method_id}/verify`

**Request:**
```json
{
  "code": "123456"
}
```

**Response:**
```json
{
  "verified": true,
  "message": "TOTP verified successfully"
}
```

#### MFA Login Flow

```
1. User provides email/password
   ↓
2. Credentials validated
   ↓
3. Check if MFA required
   ↓
4. Return MFA challenge response
   ↓
5. User provides MFA code
   ↓
6. Verify code (TOTP or backup)
   ↓
7. Issue access & refresh tokens
```

#### Backup Codes

- 10 codes generated on enrollment
- One-time use only
- Regenerate anytime via API
- Useful when device unavailable

**Endpoint:** `POST /api/mfa/backup-codes/regenerate`

**Response:**
```json
{
  "backup_codes": [
    "12345678",
    "87654321",
    ...
  ]
}
```

## JWT Tokens

### Access Token

**Lifetime:** 1 hour
**Algorithm:** RS256 (asymmetric)

**Claims:**
```json
{
  "sub": "user-id",
  "tenant_id": "tenant-id",
  "email": "user@example.com",
  "roles": ["admin", "user"],
  "iat": 1234567890,
  "exp": 1234571490
}
```

### Refresh Token

**Lifetime:** 30 days
**Algorithm:** RS256

**Usage:**
```
POST /api/auth/refresh
Authorization: Bearer {refresh_token}

Response:
{
  "access_token": "new-token",
  "refresh_token": "new-refresh-token",
  "expires_in": 3600
}
```

## Email Verification

### Flow

```
1. User registers
   ↓
2. Verification token generated (UUID)
   ↓
3. Email sent with verification link
   ↓
4. User clicks link: /api/verify-email?token={token}
   ↓
5. Token validated (24hr expiry)
   ↓
6. User marked as verified
   ↓
7. Redirect to success page
```

### Resend Verification

**Endpoint:** `POST /api/auth/resend-verification`

**Headers:**
```
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "message": "Verification email sent"
}
```

## Password Reset

### Request Reset

**Endpoint:** `POST /api/auth/forgot-password`

**Request:**
```json
{
  "tenant_id": "acme-corp",
  "email": "user@example.com"
}
```

**Features:**
- Rate limited (3 requests/hour)
- Always returns success (security)
- Token expires in 1 hour

### Complete Reset

**Endpoint:** `POST /api/auth/reset-password`

**Request:**
```json
{
  "token": "reset-token",
  "new_password": "NewSecurePassword123!"
}
```

**Response:**
```json
{
  "message": "Password reset successful"
}
```

## Session Management

### Session Storage

- Sessions stored in Redis
- Session ID in JWT claims
- TTL matches access token expiry
- Automatic cleanup on expiry

### Logout

**Endpoint:** `POST /api/auth/logout`

**Headers:**
```
Authorization: Bearer {access_token}
```

**Actions:**
1. Invalidate access token
2. Delete session from Redis
3. Revoke refresh token

## Security Policies

### Tenant-Level Configuration

**Endpoint:** `POST /api/tenants/{tenant_id}/security`

**Request:**
```json
{
  "password_min_length": 12,
  "require_mfa": true,
  "require_email_verification": true,
  "max_failed_attempts": 5,
  "lockout_duration_minutes": 30,
  "enforce_oidc": false
}
```

### MFA Enforcement

When MFA is enforced:
- All users must enroll MFA
- Login requires MFA verification
- Backup codes provided
- Grace period configurable

**Endpoint:** `POST /api/tenants/{tenant_id}/enforce-mfa`

**Request:**
```json
{
  "enforce": true,
  "grace_period_days": 7
}
```

### OIDC Enforcement

When OIDC is enforced:
- Password login disabled
- Only OIDC providers allowed
- Users must authenticate via OIDC

## Rate Limiting

### Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| Register | 5 | 1 minute |
| Login | 10 | 1 minute |
| Password Reset | 3 | 1 hour |
| Verification Resend | 3 | 1 hour |

### Headers

```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 7
X-RateLimit-Reset: 1234567890
```

## Error Responses

### Standard Format

```json
{
  "error": "error_code",
  "message": "Human-readable error message"
}
```

### Common Errors

| Status | Error Code | Description |
|--------|-----------|-------------|
| 401 | unauthorized | Invalid or missing token |
| 401 | invalid_credentials | Wrong email/password |
| 403 | mfa_required | MFA verification needed |
| 403 | account_locked | Too many failed attempts |
| 403 | email_not_verified | Email verification required |
| 429 | rate_limit_exceeded | Too many requests |

## Best Practices

### For Developers

1. **Always use HTTPS** in production
2. **Store tokens securely** (httpOnly cookies or secure storage)
3. **Implement token refresh** before expiry
4. **Handle MFA challenges** gracefully
5. **Never log tokens** or secrets
6. **Validate all inputs** on client and server

### For Administrators

1. **Enable MFA enforcement** for sensitive tenants
2. **Use OIDC** for enterprise SSO
3. **Configure strong password policies**
4. **Monitor failed login attempts**
5. **Regular security policy reviews**
6. **Rotate secrets** periodically
