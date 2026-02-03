# Multi-Tenant Architecture

## Overview

CoreAuth implements a **3-tier multi-tenant CIAM** system where your customers (tenants) can manage authentication for their own customers' organizations.

## Hierarchy Structure

```
┌─────────────────────────────────────────────────┐
│ Platform (CoreAuth)                             │
│ - Platform Admins                               │
│ - Global Configuration                          │
└──────────────────┬──────────────────────────────┘
                   │
    ┌──────────────┴──────────────┐
    │                             │
┌───▼─────────────────┐  ┌────────▼──────────────┐
│ Tenant A (Acme Inc) │  │ Tenant B (Beta Corp)  │
│ - Tenant Admins     │  │ - Tenant Admins       │
│ - Applications      │  │ - Applications        │
│ - Connections       │  │ - Connections         │
│ - Actions/Hooks     │  │ - Actions/Hooks       │
└──────────┬──────────┘  └────────┬──────────────┘
           │                      │
    ┌──────┴─────┐         ┌──────┴──────┐
    │            │         │             │
┌───▼────┐  ┌───▼────┐ ┌──▼─────┐  ┌───▼────┐
│ Org 1  │  │ Org 2  │ │ Org 1  │  │ Org 2  │
│ Users  │  │ Users  │ │ Users  │  │ Users  │
└────────┘  └────────┘ └────────┘  └────────┘
```

## Tier Definitions

### Tier 1: Platform (CoreAuth)
- **Who**: CoreAuth itself
- **Purpose**: Provide CIAM infrastructure
- **Capabilities**:
  - Manage all tenants
  - Platform-level monitoring
  - Global security policies
  - System configuration

### Tier 2: Tenant (Your Customers)
- **Who**: Companies using CoreAuth (e.g., "Acme Inc", "Beta Corp")
- **Purpose**: Use CoreAuth to manage authentication for their customers
- **Capabilities**:
  - Create and manage customer organizations
  - Configure OAuth applications
  - Set up SSO connections (OIDC/SAML)
  - Create custom actions/hooks
  - View audit logs
  - Manage billing (future)

**Examples:**
- A SaaS company using CoreAuth to handle auth for their B2B customers
- An enterprise using CoreAuth for employee + customer authentication
- A platform using CoreAuth for their marketplace vendors

### Tier 3: Organization (Tenant's Customers)
- **Who**: End customers of the tenant
- **Purpose**: The actual users being authenticated
- **Capabilities**:
  - User management
  - Role assignments
  - Organization-specific settings
  - SSO configuration (inherited or own)

**Examples:**
- "Acme Inc" (tenant) has customers: "Company A", "Company B", "Company C"
- Each company is an organization with its own users

## Signup & Onboarding Flow

### Tenant Signup (Your Customers)

```
1. Visit CoreAuth → "Sign Up"
2. Fill tenant registration:
   - Company Name: "Acme Inc"
   - Tenant Slug: "acme"
   - Admin Email: admin@acme.com
   - Admin Password: ********
3. Verify email via Mailhog
4. Login redirects to Tenant Dashboard
5. Tenant sees:
   - Customer Organizations (0)
   - Applications (0)
   - Connections (0)
   - Actions (0)
```

### Organization Creation (Tenant's Customers)

```
1. Tenant logs into CoreAuth dashboard
2. Navigate to "Customer Organizations"
3. Click "Create Organization"
4. Fill form:
   - Organization Name: "Company A"
   - Organization Slug: "company-a"
   - Parent Organization: None (or select parent)
5. Organization created
6. Tenant can now:
   - Invite users to this organization
   - Assign roles
   - Configure SSO
   - Apply custom actions
```

## Database Schema

### Organizations Table (Handles Both Tenants & Organizations)

```sql
CREATE TABLE organizations (
    id UUID PRIMARY KEY,
    parent_organization_id UUID REFERENCES organizations(id),
    slug VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,

    -- Hierarchy
    hierarchy_level INTEGER NOT NULL DEFAULT 0,
    hierarchy_path TEXT NOT NULL,

    -- Settings
    settings JSONB,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Hierarchy Rules:**
- `hierarchy_level = 0` → Tenant (root organization)
- `hierarchy_level = 1` → Customer organization under tenant
- **Max depth = 2** (Tenant → Organization only)

### Applications Table

```sql
CREATE TABLE applications (
    id UUID PRIMARY KEY,
    organization_id UUID REFERENCES organizations(id), -- Tenant ID
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    client_id VARCHAR(255) UNIQUE NOT NULL,
    client_secret_hash TEXT,

    -- OAuth Configuration
    app_type VARCHAR(50) NOT NULL,
    callback_urls TEXT[],
    logout_urls TEXT[],
    web_origins TEXT[],

    -- Scoping
    require_organization BOOLEAN DEFAULT false,
    is_enabled BOOLEAN DEFAULT true,

    UNIQUE(organization_id, slug)
);
```

**Key Points:**
- Each tenant has its own applications
- Applications are isolated per tenant
- Slug uniqueness is per-tenant, not global

### Actions Table

```sql
CREATE TABLE actions (
    id UUID PRIMARY KEY,
    organization_id UUID REFERENCES organizations(id) NOT NULL, -- Tenant ID
    name VARCHAR(255) NOT NULL,
    trigger_type VARCHAR(50) NOT NULL,
    code TEXT NOT NULL,

    -- Execution
    runtime VARCHAR(50) DEFAULT 'nodejs18',
    timeout_seconds INTEGER DEFAULT 10,
    secrets JSONB DEFAULT '{}',
    execution_order INTEGER DEFAULT 0,
    is_enabled BOOLEAN DEFAULT true,

    -- Stats
    total_executions BIGINT DEFAULT 0,
    total_failures BIGINT DEFAULT 0,
    last_executed_at TIMESTAMPTZ
);
```

**Key Points:**
- Actions belong to tenants
- Execute during auth flows for any organization under that tenant
- Can access organization context in JavaScript code

## API Structure

### Tenant Endpoints

```
POST   /api/tenants                    - Create tenant (public signup)
GET    /api/tenants/:tenant_id         - Get tenant details
PUT    /api/tenants/:tenant_id         - Update tenant
DELETE /api/tenants/:tenant_id         - Delete tenant
```

### Organization Endpoints (Tenant's Customers)

```
GET    /api/organizations                           - List customer organizations
POST   /api/organizations                           - Create customer organization
GET    /api/organizations/:org_id                   - Get organization
PUT    /api/organizations/:org_id                   - Update organization
DELETE /api/organizations/:org_id                   - Delete organization
GET    /api/organizations/:parent_id/organizations - List child organizations
```

### Application Endpoints (Tenant-scoped)

```
GET    /api/organizations/:tenant_id/applications           - List apps
POST   /api/organizations/:tenant_id/applications           - Create app
GET    /api/organizations/:tenant_id/applications/:app_id   - Get app
PUT    /api/organizations/:tenant_id/applications/:app_id   - Update app
DELETE /api/organizations/:tenant_id/applications/:app_id   - Delete app
POST   /api/organizations/:tenant_id/applications/:app_id/rotate-secret
```

Note: `:tenant_id` in URL is actually an organization_id where hierarchy_level = 0

### Action Endpoints (Tenant-scoped)

```
GET    /api/organizations/:tenant_id/actions              - List actions
POST   /api/organizations/:tenant_id/actions              - Create action
GET    /api/organizations/:tenant_id/actions/:action_id   - Get action
PUT    /api/organizations/:tenant_id/actions/:action_id   - Update action
DELETE /api/organizations/:tenant_id/actions/:action_id   - Delete action
POST   /api/organizations/:tenant_id/actions/:action_id/test - Test action
GET    /api/organizations/:tenant_id/actions/:action_id/executions
```

## Authentication Flow

### Hierarchical Login

```rust
POST /api/auth/login-hierarchical
{
    "email": "user@company-a.com",
    "password": "password123",
    "organization_slug": "company-a"  // Customer organization
}
```

**Process:**
1. Find organization by slug (`company-a`)
2. Find user by email in that organization
3. Verify password
4. Execute pre-login actions (from parent tenant)
5. Check MFA requirements
6. Generate JWT with organization context
7. Execute post-login actions
8. Return access + refresh tokens

### JWT Claims Structure

```json
{
  "sub": "user-uuid",
  "email": "user@company-a.com",
  "organization_id": "company-a-uuid",
  "tenant_id": "acme-tenant-uuid",
  "roles": ["member"],
  "exp": 1234567890,
  "iat": 1234567890
}
```

## Frontend Pages

### For Tenants (Your Customers)

1. **Dashboard** (`/dashboard`)
   - Quick stats
   - Recent activity
   - Quick actions

2. **Customer Organizations** (`/organizations`)
   - List all customer organizations
   - Create new organizations
   - Manage hierarchy (max 2 levels)

3. **Applications** (`/applications`)
   - OAuth application management
   - Client ID/Secret
   - Callback URLs configuration

4. **SSO Connections** (`/connections`)
   - OIDC/SAML provider setup
   - Test connection
   - Enable/disable

5. **Actions & Hooks** (`/actions`)
   - JavaScript code editor
   - Trigger configuration
   - Test with sample data
   - Execution logs

### For Organizations (Tenant's Customers)

Organizations don't directly access CoreAuth. They:
- Login via tenant's application
- Use SSO configured by tenant
- Have users managed by tenant admin
- See branding/UX of the tenant

## Use Case Examples

### Example 1: SaaS Company (Tenant)

**Tenant:** "CloudDocs Inc"
**Slug:** `clouddocs`

**Customer Organizations:**
- Company A (slug: `company-a`)
  - 50 employees
  - Google SSO
  - Custom role: "Document Admin"
- Company B (slug: `company-b`)
  - 200 employees
  - Azure AD SSO
  - Custom role: "Reviewer"

**Applications:**
- CloudDocs Web App
  - client_id: `clouddocs_web_xyz`
  - Requires organization selection
- CloudDocs Mobile API
  - client_id: `clouddocs_mobile_abc`
  - M2M authentication

**Actions:**
- Post-Login: Add custom claims for document permissions
- Pre-Registration: Validate email domain against allow-list

### Example 2: Enterprise (Tenant)

**Tenant:** "MegaCorp International"
**Slug:** `megacorp`

**Customer Organizations:**
- Employees (slug: `employees`)
  - Internal staff
  - Azure AD SSO
- Partners (slug: `partners`)
  - External vendors
  - SAML SSO
- Customers (slug: `customers`)
  - End customers
  - Email/password + Google OAuth

**Applications:**
- Employee Portal
- Partner Portal
- Customer Portal

**Actions:**
- Pre-Login: Check if user is in allowed region
- Post-Login: Log to security SIEM

## Security Considerations

### Tenant Isolation

- All data is tenant-scoped
- Applications cannot see other tenants
- Actions execute in sandbox per tenant
- Audit logs are tenant-specific

### Organization Isolation

- Users belong to one organization
- Cross-organization access requires explicit grants
- SSO configs are organization-specific
- Roles are scoped to organization

### JWT Security

- Short-lived access tokens (1 hour)
- Long-lived refresh tokens (30 days)
- Include both organization_id and tenant_id
- Validate on every request

## Migration from Legacy

If you have existing `tenants` table:

```sql
-- tenants table IS organizations table with hierarchy_level = 0
-- No separate table needed

-- View for backward compatibility
CREATE VIEW tenants AS
SELECT * FROM organizations
WHERE hierarchy_level = 0;
```

## Future Enhancements

1. **Social Tenant Signup**
   - Sign up with Google/GitHub
   - Auto-create tenant
   - Simplified onboarding

2. **Tenant Marketplace**
   - Pre-built action templates
   - SSO provider configurations
   - Integration apps

3. **Multi-Region Support**
   - Data residency options
   - Regional tenants
   - GDPR compliance

4. **Advanced Hierarchy**
   - Support 3+ levels (optional)
   - Business units
   - Departments

5. **Tenant Billing**
   - Usage-based pricing
   - Organization limits
   - Feature gates

## References

- [Auth0 Architecture](https://auth0.com/docs/get-started/architecture-scenarios)
- [WorkOS Organizations](https://workos.com/docs/organizations)
- [Clerk Multi-Tenancy](https://clerk.com/docs/organizations/overview)
