# CoreAuth SDK Integration Guide

## Overview

CoreAuth supports two integration patterns for your applications:

1. **Redirect-based (Hosted Login)** - Users go to CoreAuth for authentication
2. **SDK-based (Embedded Login)** - Users stay on your app, SDK handles auth

## Integration Patterns

### Pattern 1: Redirect-based (Recommended)

Users are redirected to CoreAuth's Universal Login page, then back to your app with an authorization code.

**Advantages:**

- Most secure (credentials never touch your app)
- No login UI to build
- Automatic MFA handling
- SSO integration built-in

**Flow:**

```text
User clicks "Login"
  -> Redirect to CoreAuth (/authorize)
  -> User logs in (email/password or SSO)
  -> Redirect back to your app with auth code
  -> Your app exchanges code for tokens
  -> User is logged in
```

**Implementation:**

```text
GET /authorize
  ?client_id=YOUR_CLIENT_ID
  &redirect_uri=https://yourapp.com/callback
  &response_type=code
  &scope=openid profile email offline_access
  &state=random-csrf-token
```

---

### Pattern 2: SDK-based (Embedded)

Users stay on your app. The SDK communicates with CoreAuth API in the background.

**Advantages:**

- Seamless UX (no redirect)
- Full UI control
- Custom branding
- Mobile-friendly

**Considerations:**

- You must build the login UI
- You handle MFA flows
- Credentials pass through your app

---

## Official SDKs

### Node.js / TypeScript

```bash
npm install @coreauth/sdk
```

```typescript
import { CoreAuth } from '@coreauth/sdk';

const auth = new CoreAuth({
  baseUrl: 'http://localhost:8000',
  clientId: 'your-client-id',
  clientSecret: 'your-client-secret',
});

// Login
const result = await auth.login({
  email: 'user@acme.com',
  password: 'password',
  tenantId: 'tenant-uuid',
});

// Get user
const user = await auth.getUser(result.access_token);

// Refresh token
const refreshed = await auth.refreshToken(result.refresh_token);
```

### Python

```bash
pip install coreauth
```

```python
from coreauth import CoreAuth

auth = CoreAuth(
    base_url="http://localhost:8000",
    client_id="your-client-id",
    client_secret="your-client-secret",
)

# Login
result = auth.login(
    email="user@acme.com",
    password="password",
    tenant_id="tenant-uuid",
)

# Get user
user = auth.get_user(result.access_token)
```

### Go

```go
import "github.com/cloudomate/coreauth/sdk/go/coreauth"

client := coreauth.NewClient(coreauth.Config{
    BaseURL:      "http://localhost:8000",
    ClientID:     "your-client-id",
    ClientSecret: "your-client-secret",
})

// Login
result, err := client.Login(ctx, coreauth.LoginRequest{
    Email:    "user@acme.com",
    Password: "password",
    TenantID: "tenant-uuid",
})

// Get user
user, err := client.GetUser(ctx, result.AccessToken)
```

---

## OAuth2 Authorization Code Flow

For server-side web applications, use the standard OAuth2 authorization code flow.

### Step 1: Redirect to CoreAuth

```text
GET http://localhost:8000/authorize
  ?client_id=YOUR_CLIENT_ID
  &redirect_uri=http://localhost:3000/callback
  &response_type=code
  &scope=openid profile email offline_access
  &state=random-csrf-token
```

### Step 2: Handle the Callback

After login, CoreAuth redirects to your callback URL:

```text
GET http://localhost:3000/callback?code=AUTH_CODE&state=random-csrf-token
```

### Step 3: Exchange Code for Tokens

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

### Step 4: Get User Info

```bash
curl http://localhost:8000/userinfo \
  -H "Authorization: Bearer ACCESS_TOKEN"
```

---

## Proxy-Based Integration

The CoreAuth Proxy (`coreauth-proxy`) provides a zero-code integration option. It sits in front of your application and injects identity headers.

```text
Browser -> CoreAuth Proxy (:4000) -> Your App (:3001)
```

Your app reads identity from headers:

```javascript
app.get('/dashboard', (req, res) => {
  const userId = req.headers['x-coreauth-user-id'];
  const email = req.headers['x-coreauth-user-email'];
  const tenantId = req.headers['x-coreauth-tenant-id'];
  const role = req.headers['x-coreauth-role'];
  // No JWT validation needed - the proxy handles it
});
```

The proxy handles OAuth2 sessions, token refresh, and logout automatically. See `samples/corerun-auth/` for a complete working example.

---

## Backend Token Validation

If not using the proxy, validate tokens in your backend:

### Node.js

```javascript
const jwt = require('jsonwebtoken');

function requireAuth(req, res, next) {
  const token = req.headers.authorization?.split(' ')[1];
  if (!token) return res.status(401).json({ error: 'No token' });

  try {
    const decoded = jwt.verify(token, JWT_PUBLIC_KEY);
    req.user = {
      id: decoded.sub,
      email: decoded.email,
      tenantId: decoded.tenant_id,
      role: decoded.role,
    };
    next();
  } catch {
    return res.status(401).json({ error: 'Invalid token' });
  }
}
```

### Get Public Key

```bash
curl http://localhost:8000/.well-known/jwks.json
```

---

## OIDC Discovery

CoreAuth implements OpenID Connect Discovery:

```bash
curl http://localhost:8000/.well-known/openid-configuration
```

Use this with any standard OIDC client library for automatic configuration.

---

## Sample Application

The `samples/corerun-auth/` directory contains a fully working Express.js application demonstrating:

- OAuth2/OIDC login via CoreAuth
- User and group management
- Security settings (MFA, password policies)
- Branding management
- FGA (Fine-Grained Authorization) integration
- Proxy-based header authentication

```bash
cd samples/corerun-auth
npm install
npm run dev
```

---

## Troubleshooting

### CORS error

Add your app URL to `CORS_ORIGINS` in `.env`:

```env
CORS_ORIGINS=http://localhost:3000,http://localhost:3001,https://yourapp.com
```

### Invalid token

- Check token expiration
- Verify JWT signature with the JWKS endpoint
- Ensure the token was issued for the correct audience

### Organization not found

Verify the organization slug or tenant ID matches the database.

### Redirect URI mismatch

Ensure the redirect URI matches exactly what was registered in the application configuration, including trailing slashes and protocol.
