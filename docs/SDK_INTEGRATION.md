# CoreAuth SDK Integration Guide

## Overview

CoreAuth supports **two integration patterns** for your applications:

1. **🔄 Redirect-based (Hosted Login)** - Users go to CoreAuth for authentication
2. **📱 SDK-based (Embedded Login)** - Users stay on your app, SDK handles auth

## Integration Patterns

### Pattern 1: Redirect-based (Recommended)

Users are redirected to CoreAuth's login page, then back to your app.

**✅ Advantages:**
- Most secure (credentials never touch your app)
- No UI to build
- Automatic MFA handling
- Easy SSO integration
- Works across all platforms

**Flow:**
```
User clicks "Login" on Jellp
    ↓
Redirects to CoreAuth (auth.imys.com/login)
    ↓
User logs in (email/password or SSO)
    ↓
Redirects back to Jellp with auth code
    ↓
Jellp exchanges code for tokens
    ↓
User logged into Jellp
```

**Implementation:**
```javascript
// Simple redirect
window.location.href = 'http://localhost:8000/api/oidc/login?' +
  'organization=acme&' +
  'redirect_uri=https://jellp.app/callback';
```

---

### Pattern 2: SDK-based (Embedded)

Users stay on your app, SDK communicates with CoreAuth API in the background.

**✅ Advantages:**
- Seamless UX (no redirect)
- Full UI control
- Custom branding
- Mobile app friendly

**⚠️ Considerations:**
- Must build login UI
- Handle MFA flows yourself
- Credentials pass through your app
- Requires more code

**Implementation:**
```javascript
import CoreAuth from '@coreauth/js-sdk';

const auth = new CoreAuth({ domain: 'http://localhost:8000' });

// Login with credentials
const result = await auth.loginWithCredentials(email, password, org);

if (result.requiresMFA) {
  // Handle MFA enrollment
  const mfa = await auth.enrollMFA(result.enrollmentToken);
  // Show QR code, get code from user
  await auth.verifyMFA(result.enrollmentToken, mfa.method_id, code);
}
```

---

## For imys/Jellp: Recommended Approach

### Hybrid Approach (Best of Both Worlds)

Use **redirect for SSO**, **embedded for email/password**.

```javascript
const auth = new CoreAuth({
  domain: 'https://auth.imys.com',
  clientId: 'jellp-app',
  redirectUri: 'https://jellp.app/callback'
});

// Email-first flow
async function handleLogin(email) {
  const domain = email.split('@')[1];

  // Detect organization from email
  const { organization_slug, has_sso } = await detectOrganization(email);

  if (has_sso) {
    // Customer has SSO configured → Use redirect
    auth.loginWithSSO(organization_slug);
  } else {
    // No SSO → Use embedded email/password
    showPasswordForm(email, organization_slug);
  }
}
```

**Why this works:**
- **Acme (has Okta SSO)** → Redirects to Okta → Best UX for enterprise
- **Small customer (no SSO)** → Email/password on Jellp → No jarring redirect
- **imys admins (Entra ID)** → Redirects to Microsoft → Professional SSO flow

---

## SDK Installation & Usage

### JavaScript SDK

**Installation:**
```bash
# Download
wget https://raw.githubusercontent.com/yourorg/coreauth/main/sdks/javascript/coreauth-js-sdk.js

# Include in HTML
<script src="./coreauth-js-sdk.js"></script>
```

**Basic Usage:**
```javascript
// Initialize
const auth = new CoreAuth({
  domain: 'http://localhost:8000',
  clientId: 'jellp-app',
  redirectUri: 'http://localhost:3001/callback'
});

// Login
await auth.loginWithCredentials(email, password, organization);

// Get user
const user = await auth.getUser();

// Logout
await auth.logout();
```

**See full documentation:** [sdks/javascript/README.md](../sdks/javascript/README.md)

---

## Backend Token Validation

Your backend (Jellp API) needs to validate CoreAuth tokens.

### Node.js Example

```javascript
const jwt = require('jsonwebtoken');

// Middleware to validate tokens
function requireAuth(req, res, next) {
  const token = req.headers.authorization?.split(' ')[1];

  if (!token) {
    return res.status(401).json({ error: 'No token provided' });
  }

  try {
    // Verify token with CoreAuth public key
    const decoded = jwt.verify(token, JWT_PUBLIC_KEY);

    // Attach user info to request
    req.user = {
      id: decoded.sub,
      email: decoded.email,
      organizationId: decoded.organization_id,
      role: decoded.role
    };

    next();
  } catch (err) {
    return res.status(401).json({ error: 'Invalid token' });
  }
}

// Protected route
app.get('/api/user/profile', requireAuth, (req, res) => {
  // req.user contains authenticated user info
  res.json({ user: req.user });
});
```

### Get CoreAuth Public Key

```bash
# Get public key for JWT verification
curl http://localhost:8000/.well-known/jwks.json
```

---

## Example Implementations

### 1. Jellp Login Page (Hybrid)

See: [sdks/javascript/examples/jellp-integration.html](../sdks/javascript/examples/jellp-integration.html)

**Features:**
- Email-first login
- Auto-detects organization from email domain
- Shows SSO button if available
- Falls back to password if no SSO
- Handles MFA enrollment

**Try it:**
```bash
# Open in browser
open sdks/javascript/examples/jellp-integration.html
```

### 2. React App Integration

```javascript
import { createContext, useContext, useState, useEffect } from 'react';
import CoreAuth from '@coreauth/js-sdk';

const AuthContext = createContext();

export function AuthProvider({ children }) {
  const [auth] = useState(() => new CoreAuth({
    domain: process.env.REACT_APP_AUTH_DOMAIN,
    clientId: process.env.REACT_APP_CLIENT_ID,
    redirectUri: `${window.location.origin}/callback`
  }));

  const [user, setUser] = useState(null);

  useEffect(() => {
    auth.getUser().then(setUser).catch(() => setUser(null));
  }, [auth]);

  return (
    <AuthContext.Provider value={{ auth, user, setUser }}>
      {children}
    </AuthContext.Provider>
  );
}

export const useAuth = () => useContext(AuthContext);

// Usage in components
function LoginPage() {
  const { auth } = useAuth();

  const handleLogin = async (email, password, org) => {
    const result = await auth.loginWithCredentials(email, password, org);
    if (!result.requiresMFA) {
      navigate('/dashboard');
    }
  };

  return <LoginForm onSubmit={handleLogin} />;
}
```

### 3. Mobile App (React Native)

```javascript
import CoreAuth from '@coreauth/js-sdk';
import AsyncStorage from '@react-native-async-storage/async-storage';

// Custom storage adapter for React Native
class RNStorageAdapter {
  async getItem(key) {
    return AsyncStorage.getItem(key);
  }
  async setItem(key, value) {
    return AsyncStorage.setItem(key, value);
  }
  async removeItem(key) {
    return AsyncStorage.removeItem(key);
  }
}

const auth = new CoreAuth({
  domain: 'https://auth.imys.com',
  storage: new RNStorageAdapter()
});

// Login in React Native
async function login(email, password, org) {
  const result = await auth.loginWithCredentials(email, password, org);

  if (result.success) {
    navigation.navigate('Home');
  }
}
```

---

## API Endpoints for Integration

### For Jellp Application

Your Jellp app can use these CoreAuth endpoints:

**Authentication:**
```
POST /api/auth/login-hierarchical
POST /api/auth/refresh
POST /api/auth/logout
GET  /api/auth/me
```

**OIDC/SSO:**
```
GET  /api/oidc/login
GET  /api/oidc/callback
```

**Organization Resolution:**
```
POST /api/auth/resolve-org
Body: { "email": "user@acme.com", "domain": "acme.com" }
Response: { "organization_slug": "acme", "has_sso": true }
```

**User Management:**
```
GET  /api/organizations/:org_id/users
POST /api/organizations/:org_id/users/invite
```

---

## Security Best Practices

### 1. Token Storage

**Browser:**
```javascript
// Use localStorage for SPAs
localStorage.setItem('access_token', token);

// Use httpOnly cookies for server-rendered apps (more secure)
document.cookie = `token=${token}; HttpOnly; Secure; SameSite=Strict`;
```

**Mobile:**
```javascript
// Use secure storage (Keychain on iOS, KeyStore on Android)
import SecureStorage from 'react-native-secure-storage';
await SecureStorage.setItem('access_token', token);
```

### 2. Token Refresh

```javascript
// Automatic token refresh with SDK
const authenticatedFetch = auth.createAuthenticatedFetch();

// SDK handles refresh automatically on 401
const response = await authenticatedFetch('/api/data');
```

### 3. CORS Configuration

Update CoreAuth `.env`:
```env
CORS_ORIGINS=https://jellp.app,https://jellp.app:3000,http://localhost:3001
```

### 4. HTTPS in Production

```env
# Production environment
FRONTEND_URL=https://jellp.app
BACKEND_URL=https://auth.imys.com

# Redirect URIs
http://localhost:3001/callback  # Development
https://jellp.app/callback       # Production
```

---

## Testing Your Integration

### 1. Test Email/Password Login

```bash
curl -X POST http://localhost:8000/api/auth/login-hierarchical \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@acme.com",
    "password": "password123",
    "organization_slug": "acme"
  }'
```

### 2. Test Token Validation

```bash
curl http://localhost:8000/api/auth/me \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

### 3. Test SSO Flow

```bash
# Visit in browser
http://localhost:8000/api/oidc/login?organization=acme&redirect_uri=http://localhost:3001/callback
```

---

## Troubleshooting

### Issue: "CORS error"

**Solution:** Add your app URL to `CORS_ORIGINS` in `.env`

### Issue: "Invalid token"

**Solution:** Check token expiration and verify JWT signature with public key

### Issue: "Organization not found"

**Solution:** Verify organization slug matches database:
```sql
SELECT slug FROM organizations WHERE slug = 'acme';
```

### Issue: "Redirect URI mismatch"

**Solution:** Ensure redirect URI matches exactly:
- Check OIDC provider settings
- Check CoreAuth application settings
- Include/exclude trailing slash consistently

---

## Next Steps

1. ✅ Choose integration pattern (redirect or embedded)
2. ✅ Install JavaScript SDK
3. ✅ Implement login flow
4. ✅ Test with sample credentials
5. ✅ Add SSO for enterprise customers
6. ✅ Implement token refresh
7. ✅ Setup production environment

For questions or support, check:
- [JavaScript SDK Documentation](../sdks/javascript/README.md)
- [imys Setup Guide](IMYS_SETUP_GUIDE.md)
- [API Reference](../coreauth-core/README.md)
