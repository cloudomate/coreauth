# CoreAuth Developer Guide

Build multi-tenant SaaS applications with authentication, authorization, branding, and SSO — powered by CoreAuth.

This guide walks you through the full developer journey, from deploying CoreAuth to integrating it into your application. Works for both **on-prem** (self-hosted Docker) and **SaaS** (hosted CoreAuth Cloud) deployments.

---

## Table of Contents

1. [Deploy CoreAuth](#1-deploy-coreauth)
2. [Create Your Tenant](#2-create-your-tenant)
3. [Authenticate as Tenant Admin](#3-authenticate-as-tenant-admin)
4. [Register Your Application](#4-register-your-application)
5. [Configure Branding](#5-configure-branding)
6. [Integrate OAuth2/OIDC](#6-integrate-oauth2oidc-in-your-app)
7. [Configure Security Policies](#7-configure-security-policies)
8. [Set Up SSO](#8-set-up-sso-optional)
9. [Set Up Webhooks](#9-set-up-webhooks-optional)
10. [User Management](#10-user-management)
11. [Fine-Grained Authorization](#11-fine-grained-authorization-optional)
12. [Reference: Sample App](#12-reference-sample-app)

---

## 1. Deploy CoreAuth

### Option A: On-Prem (Docker Self-Hosted)

```bash
git clone https://github.com/your-org/coreauth.git
cd coreauth
docker compose up -d
```

CoreAuth will be available at `http://localhost:8000`. Services started:

| Service    | Port  | Description              |
|------------|-------|--------------------------|
| Backend    | 8000  | CoreAuth API + Auth Server |
| Frontend   | 3000  | Admin Dashboard (React)  |
| PostgreSQL | 5432  | Database                 |
| Redis      | 6379  | Cache + Sessions         |

Verify it's running:

```bash
curl http://localhost:8000/health
```

### Option B: SaaS (CoreAuth Cloud)

Sign up at CoreAuth Cloud and note your `COREAUTH_BASE_URL` (e.g., `https://auth.your-domain.com`).

---

## 2. Create Your Tenant

Each tenant is an isolated organization with its own users, settings, branding, and security policies.

```bash
curl -X POST http://localhost:8000/api/tenants \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "Acme Corp",
    "slug": "acme",
    "admin_email": "admin@acme.com",
    "admin_password": "SecureP@ssw0rd!",
    "admin_full_name": "Admin User"
  }'
```

Response:

```json
{
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "admin_user_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "message": "Tenant created successfully"
}
```

Save the `tenant_id` — you'll need it for all subsequent API calls.

---

## 3. Authenticate as Tenant Admin

```bash
curl -X POST http://localhost:8000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "tenant_id": "YOUR_TENANT_ID",
    "email": "admin@acme.com",
    "password": "SecureP@ssw0rd!"
  }'
```

Response:

```json
{
  "status": "success",
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

Use the `access_token` as a Bearer token for authenticated API calls:

```bash
export TOKEN="eyJhbGciOiJSUzI1NiIs..."
```

---

## 4. Register Your Application

Register an OAuth2/OIDC application (client) for your SaaS app:

```bash
curl -X POST "http://localhost:8000/api/organizations/${TENANT_ID}/applications" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "My SaaS App",
    "slug": "my-saas-app",
    "app_type": "webapp",
    "callback_urls": ["http://localhost:3000/callback"],
    "logout_urls": ["http://localhost:3000"]
  }'
```

Response:

```json
{
  "id": "app_abc123...",
  "client_id": "app_0a05dc22993744dfb42cf9a840fd414f",
  "client_secret_plain": "AngRbsM1n454...keep-this-secret",
  "name": "My SaaS App"
}
```

> **Important:** Save `client_id` and `client_secret_plain` immediately. The secret is only shown once.

---

## 5. Configure Branding

Customize the look of emails (verification, password reset, invitations) and the Universal Login page.

### Get Current Branding

```bash
curl "http://localhost:8000/api/organizations/${TENANT_ID}/branding" \
  -H "Authorization: Bearer ${TOKEN}"
```

### Update Branding

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

| Field         | Description                                         | Default     |
|---------------|-----------------------------------------------------|-------------|
| `app_name`    | Display name in emails ("The Acme Corp Team")       | "CoreAuth"  |
| `logo_url`    | Logo shown in email headers and login page           | None        |
| `primary_color` | Button/link color in emails and login page (hex)  | "#2563eb"   |
| `favicon_url` | Favicon for the login page                          | None        |

Once set, all emails sent to your tenant's users will use your branding automatically — verification emails, password reset emails, invitation emails, MFA enforcement notices, and magic link emails.

---

## 6. Integrate OAuth2/OIDC in Your App

CoreAuth is a standard OAuth2/OIDC provider. Integrate using any OAuth2 library.

### Authorization Code Flow (Recommended for Web Apps)

**Step 1: Redirect to CoreAuth**

```
GET http://localhost:8000/authorize
  ?client_id=YOUR_CLIENT_ID
  &redirect_uri=http://localhost:3000/callback
  &response_type=code
  &scope=openid profile email offline_access
  &state=random-csrf-token
```

The user will see the Universal Login page with your branding (logo + colors).

**Step 2: Handle the Callback**

After login, CoreAuth redirects to your `callback_url` with an authorization code:

```
GET http://localhost:3000/callback?code=AUTH_CODE&state=random-csrf-token
```

**Step 3: Exchange Code for Tokens**

```bash
curl -X POST http://localhost:8000/oauth/token \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'grant_type=authorization_code' \
  -d 'code=AUTH_CODE' \
  -d 'redirect_uri=http://localhost:3000/callback' \
  -d 'client_id=YOUR_CLIENT_ID' \
  -d 'client_secret=YOUR_CLIENT_SECRET'
```

Response:

```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "id_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

**Step 4: Get User Info**

```bash
curl http://localhost:8000/userinfo \
  -H "Authorization: Bearer ${ACCESS_TOKEN}"
```

Response:

```json
{
  "sub": "user-uuid",
  "email": "user@acme.com",
  "name": "John Doe",
  "email_verified": true,
  "tenant_id": "tenant-uuid",
  "organization_slug": "acme",
  "role": "admin"
}
```

### Node.js Example

```javascript
const express = require('express');
const crypto = require('crypto');
const http = require('http');

const app = express();
const COREAUTH_URL = 'http://localhost:8000';
const CLIENT_ID = 'your-client-id';
const CLIENT_SECRET = 'your-client-secret';
const CALLBACK_URL = 'http://localhost:3000/callback';

// Step 1: Redirect to login
app.get('/login', (req, res) => {
  const state = crypto.randomBytes(16).toString('hex');
  req.session.oauthState = state;
  const params = new URLSearchParams({
    client_id: CLIENT_ID,
    redirect_uri: CALLBACK_URL,
    response_type: 'code',
    scope: 'openid profile email offline_access',
    state,
  });
  res.redirect(`${COREAUTH_URL}/authorize?${params}`);
});

// Step 2-3: Handle callback, exchange code
app.get('/callback', async (req, res) => {
  const { code, state } = req.query;
  // Verify state matches...
  const tokens = await exchangeCode(code); // POST /oauth/token
  req.session.accessToken = tokens.access_token;
  res.redirect('/dashboard');
});
```

See `samples/corerun-auth/` for a full working example.

---

## 7. Configure Security Policies

### Get Security Settings

```bash
curl "http://localhost:8000/api/organizations/${TENANT_ID}/security" \
  -H "Authorization: Bearer ${TOKEN}"
```

### Update Security Settings

```bash
curl -X PUT "http://localhost:8000/api/organizations/${TENANT_ID}/security" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "mfa_required": true,
    "require_email_verification": true,
    "password_min_length": 12,
    "password_require_uppercase": true,
    "password_require_number": true,
    "password_require_special": true,
    "max_login_attempts": 5,
    "lockout_duration_minutes": 15,
    "session_timeout_hours": 8
  }'
```

| Setting                    | Default | Description                                   |
|----------------------------|---------|-----------------------------------------------|
| `mfa_required`             | false   | Require MFA for all users                     |
| `mfa_grace_period_days`    | 7       | Days before MFA is enforced                   |
| `allowed_mfa_methods`      | ["totp","sms"] | Allowed MFA methods                  |
| `require_email_verification` | false | Require verified email before login          |
| `password_min_length`      | 8       | Minimum password length                       |
| `password_require_uppercase` | false | Require uppercase letter                     |
| `password_require_lowercase` | false | Require lowercase letter                     |
| `password_require_number`  | false   | Require number                                |
| `password_require_special` | false   | Require special character                     |
| `max_login_attempts`       | 5       | Failed attempts before lockout                |
| `lockout_duration_minutes` | 15      | Lockout duration                              |
| `session_timeout_hours`    | 24      | Session expiration                            |

---

## 8. Set Up SSO (Optional)

Connect external identity providers (Google, Microsoft, Okta, etc.) for enterprise SSO.

### Add an OIDC Provider

```bash
curl -X POST http://localhost:8000/api/oidc/providers \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "tenant_id": "YOUR_TENANT_ID",
    "name": "Google Workspace",
    "provider_type": "google",
    "client_id": "google-client-id.apps.googleusercontent.com",
    "client_secret": "google-secret",
    "issuer": "https://accounts.google.com",
    "authorization_endpoint": "https://accounts.google.com/o/oauth2/v2/auth",
    "token_endpoint": "https://oauth2.googleapis.com/token",
    "jwks_uri": "https://www.googleapis.com/oauth2/v3/certs"
  }'
```

### List Provider Templates

CoreAuth includes pre-configured templates for common providers:

```bash
curl http://localhost:8000/api/oidc/templates
```

Returns templates for Google, Microsoft, GitHub, Okta, and more with pre-filled endpoints.

---

## 9. Set Up Webhooks (Optional)

Subscribe to events (user signup, login, MFA enabled, etc.) and get notified via HTTP callbacks.

### Create a Webhook

```bash
curl -X POST "http://localhost:8000/api/organizations/${TENANT_ID}/webhooks" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "url": "https://your-app.com/webhooks/coreauth",
    "events": ["user.created", "user.login", "user.mfa_enabled", "user.password_changed"],
    "description": "Notify my app of user events"
  }'
```

Response includes a `signing_secret` for verifying webhook payloads.

### Available Event Types

```bash
curl http://localhost:8000/api/webhooks/event-types
```

---

## 10. User Management

### Invite Users

```bash
curl -X POST "http://localhost:8000/api/tenants/${TENANT_ID}/invitations" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "newuser@acme.com",
    "expires_in_days": 7
  }'
```

The invited user receives a branded email and can accept the invitation to create their account.

### List Users

```bash
curl "http://localhost:8000/api/tenants/${TENANT_ID}/users" \
  -H "Authorization: Bearer ${TOKEN}"
```

### Update User Role

```bash
curl -X PUT "http://localhost:8000/api/tenants/${TENANT_ID}/users/${USER_ID}/role" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{"role": "admin"}'
```

### Groups

Organize users into groups for easier management:

```bash
# Create a group
curl -X POST "http://localhost:8000/api/tenants/${TENANT_ID}/groups" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{"name": "Engineering", "slug": "engineering"}'

# Add a member
curl -X POST "http://localhost:8000/api/tenants/${TENANT_ID}/groups/${GROUP_ID}/members" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{"user_id": "USER_UUID", "role": "member"}'
```

---

## 11. Fine-Grained Authorization (Optional)

CoreAuth includes an FGA (Fine-Grained Authorization) system for relationship-based access control.

### Create an FGA Store

```bash
curl -X POST http://localhost:8000/api/fga/stores \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{"name": "my-app-authz", "description": "Authorization for My App"}'
```

### Define an Authorization Model

```bash
curl -X POST "http://localhost:8000/api/fga/stores/${STORE_ID}/models" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "dsl": "type document\n  relations\n    define owner: [user]\n    define editor: [user] or owner\n    define viewer: [user] or editor"
  }'
```

### Write Tuples

```bash
curl -X POST "http://localhost:8000/api/fga/stores/${STORE_ID}/tuples" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "writes": [
      {"object": "document:budget", "relation": "owner", "subject": "user:alice"}
    ]
  }'
```

### Check Permissions

```bash
curl -X POST "http://localhost:8000/api/fga/stores/${STORE_ID}/check" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{
    "object": "document:budget",
    "relation": "viewer",
    "subject": "user:alice"
  }'
```

---

## 12. Reference: Sample App

The `samples/corerun-auth/` directory contains a fully working Node.js application that demonstrates:

- OAuth2/OIDC login via CoreAuth
- User management (list, invite, roles)
- Group management
- Session management
- Security settings (MFA, password policies)
- **Branding management** (logo, colors, app name with live preview)
- Email verification flow
- FGA integration

### Run the Sample App

```bash
cd samples/corerun-auth
cp .env.example .env  # Edit with your client_id and client_secret
npm install
npm start
```

The sample app runs on `http://localhost:5050`.

### Key Files

| File | Description |
|------|-------------|
| `app.js` | Express app setup with session management |
| `services/coreauth.js` | CoreAuth API client (all API calls) |
| `routes/auth.js` | OAuth2 callback, email verification |
| `routes/admin.js` | User, group, session, and settings management |
| `views/settings.ejs` | Settings page with branding, MFA, and SSO controls |

---

## API Quick Reference

### Public Endpoints (No Auth)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| POST | `/api/tenants` | Create new tenant |
| GET | `/.well-known/openid-configuration` | OIDC discovery |
| GET | `/authorize` | OAuth2 authorize |
| POST | `/oauth/token` | Exchange code/refresh |
| GET | `/userinfo` | Get user info |
| GET | `/api/oidc/templates` | SSO provider templates |

### Authenticated Endpoints (Bearer Token)

| Method | Path | Description |
|--------|------|-------------|
| GET/PUT | `/api/organizations/:id/branding` | Branding settings |
| GET/PUT | `/api/organizations/:id/security` | Security policies |
| POST | `/api/organizations/:id/applications` | Register OAuth app |
| GET | `/api/tenants/:id/users` | List users |
| POST | `/api/tenants/:id/invitations` | Invite user |
| POST | `/api/oidc/providers` | Add SSO provider |
| POST | `/api/organizations/:id/webhooks` | Create webhook |
| POST | `/api/fga/stores` | Create FGA store |

For the complete API reference, see the route definitions in `coreauth-core/crates/api/src/routes.rs`.

---

## Architecture Overview

```
┌─────────────────┐     ┌──────────────────┐
│   Your App      │────>│   CoreAuth API   │
│  (any language) │<────│  (Rust/Axum)     │
│                 │     │                  │
│  OAuth2 Client  │     │  - Auth/OIDC     │
│  API Calls      │     │  - User Mgmt    │
│  Webhooks       │     │  - Branding     │
└─────────────────┘     │  - MFA/SSO      │
                        │  - FGA/Authz    │
                        │  - Webhooks     │
                        └──────┬───────────┘
                               │
                    ┌──────────┴──────────┐
                    │                     │
              ┌─────┴─────┐       ┌───────┴──────┐
              │ PostgreSQL │       │    Redis     │
              │  (Data)    │       │  (Cache)     │
              └────────────┘       └──────────────┘
```

Each tenant gets:
- Isolated users, applications, and settings
- Custom branding (logo, colors, app name)
- Independent security policies (MFA, password rules, lockout)
- Optional dedicated database (silo isolation)
- Own SSO/OIDC providers
- Own webhooks and event subscriptions
