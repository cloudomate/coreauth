# imys Multi-Tenant SaaS Setup Guide

## Scenario

imys is building **Jellp**, a multi-tenant SaaS application. They need:
1. **imys admins** to authenticate via Microsoft Entra ID (Azure AD)
2. **Jellp customers** to authenticate via their own OIDC providers

## Architecture

```
CoreAuth Platform
└── imys (Organization)
    ├── Admins → Login via Entra ID
    ├── Jellp Application
    └── Customers (Child Organizations)
        ├── Acme Corp → Login via Okta
        ├── Beta Inc → Login via Google Workspace
        └── Gamma LLC → Login via Auth0
```

## Step-by-Step Setup

### Phase 1: Create imys Organization

#### 1. Sign Up

Visit: http://localhost:3000/signup

```
Organization Name: imys
Organization Slug: imys
Admin Name: imys Admin
Admin Email: admin@imys.com
Password: [secure-password]
```

#### 2. Verify Email

Check the MailHog UI for verification email:
- http://localhost:8025 (if using local Mailhog)
- Click verification link

#### 3. Login

Visit: http://localhost:3000/login

```
Tenant Slug: imys
Email: admin@imys.com
Password: [your-password]
```

---

### Phase 2: Setup Entra ID for imys Admins

#### A. Configure in Azure Portal

1. **Navigate to Azure AD**
   - Go to https://portal.azure.com
   - Azure Active Directory → App registrations → New registration

2. **Register Application**
   ```
   Name: CoreAuth - imys
   Supported account types: Accounts in this organizational directory only
   Redirect URI (Web): http://localhost:8000/api/oidc/callback
   ```

3. **Get Application Details**
   After registration, copy:
   - **Application (client) ID**: e.g., `12345678-1234-1234-1234-123456789012`
   - **Directory (tenant) ID**: e.g., `87654321-4321-4321-4321-210987654321`

4. **Create Client Secret**
   - Go to: Certificates & secrets → New client secret
   - Description: `CoreAuth Production`
   - Expires: 24 months
   - Copy the **Secret Value** immediately (it won't show again)

5. **Configure API Permissions (Optional)**
   - API permissions → Add a permission → Microsoft Graph
   - Delegated permissions:
     - `openid`
     - `profile`
     - `email`
     - `User.Read`
   - Grant admin consent

6. **Configure Token Configuration (Optional)**
   - Token configuration → Add optional claim
   - ID token: email, preferred_username
   - Access token: email

#### B. Configure in CoreAuth

1. **Login as imys admin**
   - http://localhost:3000/login
   - Organization: `imys`

2. **Navigate to Connections**
   - Sidebar → Configuration → Connections
   - Click "Add Connection"

3. **Select Microsoft (Entra ID)**
   - Click on the Microsoft/Azure AD card

4. **Fill in Configuration**
   ```
   Connection Name: imys Admin SSO
   Client ID: [paste Application (client) ID from Azure]
   Client Secret: [paste Secret Value from Azure]
   Domain: imys.com (optional, for login hint)
   ```

5. **Copy Callback URL**
   ```
   http://localhost:8000/api/oidc/callback
   ```
   (Already configured in Azure, just verify)

6. **Save Connection**
   - Click "Create Connection"
   - Connection should appear as "Active"

#### C. Test Entra ID Login

1. **Logout** from current session
2. **Go to Login** page
3. **Enter**: Organization = `imys`
4. **Click "Sign in with SSO"** (if available)
5. **Redirect to Microsoft** login
6. **Enter** imys.com credentials
7. **Redirect back** to CoreAuth
8. **Success!** You're logged in via Entra ID

---

### Phase 3: Create Customer Organizations

Now create child organizations for each of imys's customers.

#### Option A: Via Dashboard (Recommended)

1. **Login as imys admin**
2. **Navigate to**: Organizations
3. **Click**: "Create Child Organization"
4. **Fill in details**:
   ```
   Customer Name: Acme Corporation
   Organization Slug: acme
   Admin Email: admin@acme.com
   Admin Password: [temporary-password]
   ```
5. **Click** "Create"
6. **Repeat** for each customer (Beta Inc, Gamma LLC, etc.)

#### Option B: Via API

```bash
# Get imys admin token first
TOKEN=$(curl -X POST http://localhost:8000/api/auth/login-hierarchical \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@imys.com",
    "password": "your-password",
    "organization_slug": "imys"
  }' | jq -r '.access_token')

# Create child organization
curl -X POST http://localhost:8000/api/organizations/<imys-org-id>/organizations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Acme Corporation",
    "slug": "acme",
    "admin_email": "admin@acme.com",
    "admin_password": "temporary-password-123"
  }'
```

#### Send Invitation to Customer Admin

```bash
# Invite customer admin
curl -X POST http://localhost:8000/api/tenants/<acme-org-id>/invitations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@acme.com",
    "role": "admin"
  }'
```

The customer admin will receive an email with:
- Login link
- Temporary password (if set)
- Instructions to set up their SSO

---

### Phase 4: Customer SSO Setup

Each customer can now configure their own OIDC provider.

#### Example: Acme Corp (Using Okta)

1. **Customer Admin Login**
   - Visit: http://localhost:3000/login
   - Organization: `acme`
   - Email: `admin@acme.com`
   - Password: [temporary-password]

2. **Navigate to Connections**
   - Sidebar → Configuration → Connections
   - Click "Add Connection"

3. **Select Okta**
   - Click on the Okta card

4. **Configure Okta** (in Okta Admin Console first)
   - Create new App Integration → OIDC
   - Application type: Web Application
   - Sign-in redirect URI: `http://localhost:8000/api/oidc/callback`
   - Copy Client ID and Client Secret

5. **Fill in CoreAuth**
   ```
   Connection Name: Acme Okta SSO
   Client ID: [from Okta]
   Client Secret: [from Okta]
   ```

6. **Test Login**
   - Logout
   - Login with organization = `acme`
   - Click SSO
   - Redirects to Okta
   - Login with Acme credentials
   - Redirects back to Jellp application

#### Example: Beta Inc (Using Google Workspace)

1. **Configure in Google Cloud Console**
   - Create OAuth 2.0 Client ID
   - Application type: Web application
   - Authorized redirect URI: `http://localhost:8000/api/oidc/callback`

2. **Configure in CoreAuth**
   ```
   Connection Name: Beta Google SSO
   Provider: Google
   Client ID: [from Google]
   Client Secret: [from Google]
   Domain: betainc.com
   ```

---

### Phase 5: Multi-Tenant Login Flow

#### Flow Diagram

```
User visits Jellp app
    ↓
Enter email: user@acme.com
    ↓
CoreAuth detects domain → organization = "acme"
    ↓
Redirect to Acme's OIDC provider (Okta)
    ↓
User logs in with Acme credentials
    ↓
Okta redirects back to CoreAuth
    ↓
CoreAuth creates/updates user in "acme" organization
    ↓
User logged into Jellp with Acme tenant context
```

#### Implementation in Jellp Frontend

```javascript
// Jellp login page
async function handleLogin(email) {
  // Extract domain from email
  const domain = email.split('@')[1];

  // Call CoreAuth to determine organization
  const response = await fetch('http://localhost:8000/api/auth/resolve-org', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, domain })
  });

  const { organization_slug } = await response.json();

  // Redirect to CoreAuth SSO
  window.location.href = `http://localhost:8000/api/oidc/login?` +
    `organization=${organization_slug}&` +
    `redirect_uri=${encodeURIComponent('http://jellp.app/callback')}`;
}
```

---

### Phase 6: Security Settings

Configure security policies for each organization.

#### imys Organization Settings

```bash
# Navigate to: Security
MFA Required: Yes (for admins)
Grace Period: 7 days
Password Policy:
  - Minimum length: 12 characters
  - Require uppercase: Yes
  - Require numbers: Yes
  - Require special characters: Yes
```

#### Customer Organization Settings

Each customer can configure their own:
- MFA requirements
- Password policies
- Session timeout
- Login restrictions

---

## Production Configuration

### Environment Variables

Update `.env` for production:

```env
# Database (production)
DATABASE_URL=postgresql://coreauth:STRONG_PASSWORD@prod-db:5432/coreauth

# JWT Secret (generate with: openssl rand -base64 64)
JWT_SECRET=very-long-production-secret-key-here

# Frontend URL
FRONTEND_URL=https://auth.imys.com

# Email (production SMTP)
EMAIL_PROVIDER=smtp
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
EMAIL_FROM=noreply@imys.com
```

### Azure AD Production URLs

Update Entra ID redirect URI to production:
```
https://auth.imys.com/api/oidc/callback
```

---

## Testing Checklist

- [ ] imys admin can login via Entra ID
- [ ] imys admin can create child organizations
- [ ] Customer admin receives invitation email
- [ ] Customer admin can access dashboard
- [ ] Customer admin can configure their OIDC
- [ ] Customer user can login via customer's OIDC
- [ ] Users are isolated to their organization
- [ ] MFA enrollment works for required orgs
- [ ] Password reset works
- [ ] Audit logs capture all events

---

## Troubleshooting

### Issue: "Invalid redirect URI" from Azure

**Solution**: Verify callback URL in Azure matches:
```
http://localhost:8000/api/oidc/callback
```

### Issue: "Organization not found"

**Solution**: Check organization slug is correct:
```sql
SELECT slug FROM organizations WHERE name = 'imys';
```

### Issue: "User already exists"

**Solution**: User emails must be unique per organization. Check:
```sql
SELECT email, organization_id FROM users WHERE email = 'user@example.com';
```

### Issue: SSO button not showing

**Solution**: Check connection is enabled:
```sql
SELECT provider_name, is_enabled FROM oidc_providers WHERE organization_id = '<org-id>';
```

---

## Next Steps

1. **Create Jellp Application**
   - Register OAuth2 client
   - Configure scopes and permissions
   - Implement token validation

2. **Setup Custom Branding**
   - White-label login page per customer
   - Custom logos and colors

3. **Configure Webhooks**
   - User created events
   - Login events
   - Security events

4. **Setup Monitoring**
   - Failed login attempts
   - SSO connection health
   - User activity logs

---

## Support

For questions or issues:
- Documentation: `/docs`
- Backend logs: `docker compose logs -f backend`
- Database: `docker compose exec postgres psql -U coreauth -d coreauth`
