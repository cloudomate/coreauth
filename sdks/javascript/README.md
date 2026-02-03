# CoreAuth JavaScript SDK

Client-side JavaScript SDK for integrating CoreAuth authentication into your web applications.

## Installation

### Via Script Tag

```html
<script src="https://cdn.yourdomain.com/coreauth-sdk.js"></script>
<script>
  const auth = new CoreAuth({
    domain: 'http://localhost:8000',
    clientId: 'your-client-id',
    redirectUri: 'http://yourapp.com/callback',
    organization: 'your-org-slug'
  });
</script>
```

### Via NPM (when published)

```bash
npm install @coreauth/js-sdk
```

```javascript
import CoreAuth from '@coreauth/js-sdk';

const auth = new CoreAuth({
  domain: 'https://auth.yourdomain.com',
  clientId: 'your-client-id',
  redirectUri: 'https://yourapp.com/callback'
});
```

## Quick Start

### Option 1: Redirect-based Login (Recommended)

```javascript
// Initialize SDK
const auth = new CoreAuth({
  domain: 'http://localhost:8000',
  clientId: 'jellp-app',
  redirectUri: 'http://localhost:3001/callback',
  organization: 'acme'
});

// Login button click
document.getElementById('login-btn').addEventListener('click', () => {
  auth.login();
});

// Handle callback
if (window.location.pathname === '/callback') {
  auth.handleCallback()
    .then(result => {
      console.log('Logged in!', result.user);
      window.location.href = '/dashboard';
    })
    .catch(err => {
      console.error('Login failed:', err);
    });
}

// Check authentication status
if (auth.isAuthenticated()) {
  auth.getUser().then(user => {
    console.log('Current user:', user);
  });
}

// Logout
document.getElementById('logout-btn').addEventListener('click', () => {
  auth.logout('/');
});
```

### Option 2: Embedded Login (Email/Password)

```javascript
const auth = new CoreAuth({
  domain: 'http://localhost:8000'
});

// Login form submit
document.getElementById('login-form').addEventListener('submit', async (e) => {
  e.preventDefault();

  const email = document.getElementById('email').value;
  const password = document.getElementById('password').value;
  const organization = document.getElementById('organization').value;

  try {
    const result = await auth.loginWithCredentials(email, password, organization);

    if (result.requiresMFA) {
      // Handle MFA enrollment
      showMFAEnrollment(result.enrollmentToken);
    } else {
      // Success - redirect to dashboard
      window.location.href = '/dashboard';
    }
  } catch (error) {
    alert('Login failed: ' + error.message);
  }
});
```

### Option 3: SSO Login

```javascript
const auth = new CoreAuth({
  domain: 'http://localhost:8000',
  redirectUri: 'http://localhost:3001/callback'
});

// SSO login button
document.getElementById('sso-btn').addEventListener('click', () => {
  auth.loginWithSSO('acme'); // Organization slug
});
```

## API Reference

### Constructor

```javascript
new CoreAuth(config)
```

**Parameters:**
- `config.domain` (string, required): CoreAuth server URL
- `config.clientId` (string, optional): OAuth2 client ID
- `config.redirectUri` (string, optional): OAuth2 redirect URI
- `config.organization` (string, optional): Default organization slug
- `config.scope` (string, optional): OAuth2 scopes (default: 'openid profile email')

### Methods

#### `login(options)`

Redirect to CoreAuth login page.

```javascript
auth.login({
  organization: 'acme',
  loginHint: 'user@acme.com'
});
```

#### `loginWithCredentials(email, password, organization)`

Login with email and password (embedded).

```javascript
const result = await auth.loginWithCredentials(
  'user@acme.com',
  'password123',
  'acme'
);

if (result.requiresMFA) {
  // Handle MFA enrollment
  const mfa = await auth.enrollMFA(result.enrollmentToken);
  // Show QR code: mfa.qr_code_uri
}
```

#### `loginWithSSO(organization, connection)`

Login with SSO provider.

```javascript
auth.loginWithSSO('acme', 'okta');
```

#### `handleCallback()`

Handle OAuth2 callback after redirect.

```javascript
const { user, accessToken } = await auth.handleCallback();
```

#### `logout(returnTo)`

Logout user and optionally redirect.

```javascript
await auth.logout('/');
```

#### `getUser()`

Get current authenticated user.

```javascript
const user = await auth.getUser();
console.log(user.email, user.metadata);
```

#### `isAuthenticated()`

Check if user is authenticated.

```javascript
if (auth.isAuthenticated()) {
  // User is logged in
}
```

#### `getAccessToken()`

Get access token.

```javascript
const token = auth.getAccessToken();
```

#### `refreshToken()`

Manually refresh access token.

```javascript
await auth.refreshToken();
```

#### `createAuthenticatedFetch()`

Create a fetch wrapper with automatic token refresh.

```javascript
const authenticatedFetch = auth.createAuthenticatedFetch();

const response = await authenticatedFetch('/api/data');
const data = await response.json();
```

## React Integration

### React Hook

```javascript
import { useState, useEffect, createContext, useContext } from 'react';
import CoreAuth from '@coreauth/js-sdk';

const AuthContext = createContext();

export function AuthProvider({ children, config }) {
  const [auth] = useState(() => new CoreAuth(config));
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    auth.getUser()
      .then(setUser)
      .catch(() => setUser(null))
      .finally(() => setLoading(false));
  }, [auth]);

  const login = (email, password, organization) => {
    return auth.loginWithCredentials(email, password, organization)
      .then(result => {
        if (!result.requiresMFA) {
          return auth.getUser().then(setUser);
        }
        return result;
      });
  };

  const logout = () => {
    return auth.logout('/login').then(() => setUser(null));
  };

  return (
    <AuthContext.Provider value={{ auth, user, loading, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
```

### Usage in React Components

```javascript
import { useAuth } from './auth';

function Dashboard() {
  const { user, logout } = useAuth();

  if (!user) {
    return <div>Not authenticated</div>;
  }

  return (
    <div>
      <h1>Welcome, {user.email}</h1>
      <button onClick={logout}>Logout</button>
    </div>
  );
}

function LoginPage() {
  const { login } = useAuth();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [org, setOrg] = useState('');

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      await login(email, password, org);
      navigate('/dashboard');
    } catch (err) {
      alert(err.message);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <input value={org} onChange={e => setOrg(e.target.value)} placeholder="Organization" />
      <input value={email} onChange={e => setEmail(e.target.value)} placeholder="Email" />
      <input type="password" value={password} onChange={e => setPassword(e.target.value)} />
      <button type="submit">Login</button>
    </form>
  );
}
```

## MFA Support

```javascript
// After login returns requiresMFA
const result = await auth.loginWithCredentials(email, password, org);

if (result.requiresMFA) {
  // 1. Enroll TOTP
  const mfa = await auth.enrollMFA(result.enrollmentToken);

  // 2. Show QR code to user
  showQRCode(mfa.qr_code_uri);

  // 3. User scans and enters code
  const code = getUserInput();

  // 4. Verify code
  await auth.verifyMFA(
    result.enrollmentToken,
    mfa.method_id,
    code
  );

  // 5. User is now logged in
  const user = await auth.getUser();
}
```

## Examples

### Jellp Application Integration

```javascript
// Jellp frontend (imys's SaaS app)
const auth = new CoreAuth({
  domain: 'https://auth.imys.com',
  clientId: 'jellp-prod',
  redirectUri: 'https://jellp.app/callback'
});

// Login page
function handleLogin() {
  const email = document.getElementById('email').value;
  const domain = email.split('@')[1];

  // Auto-detect organization from email domain
  fetch('https://auth.imys.com/api/auth/resolve-org', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, domain })
  })
  .then(res => res.json())
  .then(({ organization_slug }) => {
    // Login with detected organization
    auth.loginWithSSO(organization_slug);
  });
}
```

### Protected API Calls

```javascript
const authenticatedFetch = auth.createAuthenticatedFetch();

// Make API calls with automatic token refresh
async function fetchUserData() {
  const response = await authenticatedFetch('https://api.jellp.app/user/profile');
  return response.json();
}

async function updateProfile(data) {
  const response = await authenticatedFetch('https://api.jellp.app/user/profile', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data)
  });
  return response.json();
}
```

## Error Handling

```javascript
try {
  await auth.loginWithCredentials(email, password, org);
} catch (error) {
  if (error.message.includes('Invalid credentials')) {
    alert('Wrong email or password');
  } else if (error.message.includes('MFA required')) {
    // Handle MFA
  } else {
    alert('Login failed: ' + error.message);
  }
}
```

## Browser Support

- Chrome 60+
- Firefox 55+
- Safari 11+
- Edge 79+

## License

MIT
