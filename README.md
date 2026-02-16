<p align="center">
  <img src="logo/core-auth-logo.svg" alt="CoreAuth" width="280" />
</p>

<h3 align="center">Multi-Tenant Customer Identity & Access Management</h3>

<p align="center">
  Authentication, authorization, SSO, MFA, and fine-grained permissions for SaaS applications.
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="docs/DEVELOPER_GUIDE.md">Developer Guide</a> &middot;
  <a href="docs/ARCHITECTURE.md">Architecture</a> &middot;
  <a href="docs/SDK_INTEGRATION.md">SDK Integration</a>
</p>

---

## What is CoreAuth?

CoreAuth is a self-hosted identity platform for SaaS applications. It handles authentication, authorization, and user management so you don't have to build it yourself. Think Auth0/WorkOS, but open-source and self-hosted.

**Built with:** Rust (Axum) backend, React (Vite + Tailwind) frontend, PostgreSQL, Redis.

### Key Features

| Category | Features |
|----------|----------|
| **Authentication** | Email/password, passwordless (magic links & OTP), social login (Google, GitHub, Microsoft), OIDC/SSO |
| **Multi-Factor Auth** | TOTP (Google/Microsoft Authenticator), SMS codes, backup codes |
| **Authorization** | Fine-Grained Authorization (Zanzibar-style ReBAC), role-based access control |
| **Multi-Tenancy** | Isolated tenants with shared or dedicated databases, per-tenant branding and security policies |
| **Standards** | OAuth2 authorization server, OIDC provider, SCIM 2.0 provisioning, JWKS |
| **Developer Tools** | SDKs (Node.js, Python, Go), webhooks, audit logs, email templates, reverse proxy with identity injection |

---

## Quick Start

### Prerequisites

- Docker and Docker Compose

### Start Everything

```bash
git clone https://github.com/cloudomate/coreauth.git
cd coreauth
./start.sh
```

This starts PostgreSQL, Redis, the backend API, and the frontend dashboard.

| Service | URL |
|---------|-----|
| Frontend Dashboard | http://localhost:3000 |
| Backend API | http://localhost:8000 |
| Health Check | http://localhost:8000/health |

### Create Your First Tenant

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

See the [Developer Guide](docs/DEVELOPER_GUIDE.md) for a complete walkthrough from tenant creation through OAuth2 integration.

---

## Architecture

```
                    ┌──────────────────────────────────┐
                    │          Your Application        │
                    │    (reads X-CoreAuth-* headers)  │
                    └──────────────┬───────────────────┘
                                   │
┌──────────┐      ┌────────────────▼────────────────┐
│ Browser  │─────>│     CoreAuth Proxy (:4000)      │
└──────────┘      │  OAuth2 sessions, header inject │
                  └───────┬────────────────┬────────┘
                          │                │
              ┌───────────▼────┐  ┌──────▼──────────┐
              │  Backend AP    │  │   Frontend      │
              │  (Rust/Axum)   │  │   (React/Vite)  │
              │  :3000 (int)   │  │   :3000 (nginx) │
              │  :8000 (ext    │  │                 │
              └──────┬──────┬──┘  └─────────────────┘
                     │      │
              ┌──────▼─ ─┐ ┌─▼──────┐
              │PostgreSQL│ │ Redis  │
              │  :5432   │ │ :6379  │
              └───────── ┘ └────────┘
```

The proxy handles OAuth2 session management and injects `X-CoreAuth-*` headers (User-Id, User-Email, Tenant-Id, Role, etc.) into upstream requests. Protected applications never handle raw credentials.

### Backend Crates

```
api → auth, authz, database, models, tenant, cache
auth → models, database, cache
authz → models, database, cache
database → models
models → (standalone)
cache → (standalone, redis)
tenant → models, database, cache
```

| Crate | Purpose |
|-------|---------|
| `api` | Axum handlers, routes, middleware, AppState |
| `auth` | Login, signup, MFA, OAuth2/OIDC, email/SMS, JWT (RS256 + HS256) |
| `authz` | Fine-Grained Authorization engine (OpenFGA-compatible Zanzibar-style) |
| `database` | SQLx repositories, tenant database router, AES-GCM field encryption |
| `models` | Domain structs (serde + sqlx + validator) |
| `tenant` | Multi-tenant isolation, tenant registry, database routing |
| `cache` | Redis abstraction layer |

---

## Project Structure

```
coreauth/
├── coreauth-core/          # Rust backend (Axum API server)
│   ├── crates/             # Workspace crates (api, auth, authz, database, models, tenant, cache)
│   ├── migrations/         # SQL migrations (001-013)
│   └── Dockerfile
├── coreauth-portal/        # React frontend (Vite + Tailwind)
│   ├── src/pages/          # 21+ page components
│   └── Dockerfile
├── coreauth-proxy/         # Rust reverse proxy (identity injection, session mgmt)
│   └── src/
├── samples/
│   └── corerun-auth/       # Sample Express.js SaaS app with FGA demo
├── sdk/
│   ├── node/               # TypeScript SDK (@coreauth/sdk)
│   ├── python/             # Python SDK (coreauth)
│   └── go/                 # Go SDK
├── deploy/
│   ├── helm/               # Kubernetes Helm charts
│   └── onprem/             # On-premise Docker Compose
├── tests/                  # Integration test suites (bash)
├── docs/                   # Documentation
├── docker-compose.yml
└── start.sh                # One-command startup script
```

---

## Development

### Backend (Rust)

```bash
cd coreauth-core
cargo check                              # Fast compilation check
SQLX_OFFLINE=true cargo check            # When DB is not available
cargo build --release --locked           # Production build
cargo test                               # Run tests
```

### Frontend (React)

```bash
cd coreauth-portal
npm install
npm run dev                              # Dev server with hot reload
npm run build                            # Production build
```

### Proxy (Rust)

```bash
cd coreauth-proxy
cargo build --release                    # Binary: coreauth-proxy
```

### Sample App

```bash
cd samples/corerun-auth
npm install
npm run dev                              # Dev server on port 3001
```

---

## SDKs

Official SDKs for integrating your application with CoreAuth:

| SDK | Install | Status |
|-----|---------|--------|
| **Node.js/TypeScript** | `npm install @coreauth/sdk` | Available |
| **Python** | `pip install coreauth` | Available |
| **Go** | `go get github.com/cloudomate/coreauth/sdk/go` | Available |

See the [SDK Integration Guide](docs/SDK_INTEGRATION.md) for usage examples.

---

## Documentation

| Document | Description |
|----------|-------------|
| [Getting Started](docs/GETTING_STARTED.md) | Quick start guide |
| [Developer Guide](docs/DEVELOPER_GUIDE.md) | Full walkthrough: tenant creation through OAuth2 integration |
| [Architecture](docs/ARCHITECTURE.md) | System architecture and design |
| [Authentication](docs/AUTHENTICATION.md) | Auth methods and flows |
| [Authorization](docs/AUTHORIZATION.md) | FGA/ReBAC permission model |
| [Multi-Tenant Architecture](docs/MULTI_TENANT_ARCHITECTURE.md) | Tenant isolation and routing |
| [Features](docs/FEATURES.md) | Complete feature list |
| [Deployment](docs/DEPLOYMENT.md) | Docker, Kubernetes, cloud deployment |
| [Testing](docs/TESTING.md) | Testing guide |
| [Email & SMS Setup](docs/EMAIL_SMS_SETUP.md) | Email/SMS provider configuration |
| [SDK Integration](docs/SDK_INTEGRATION.md) | SDK usage and integration patterns |

---

## API Overview

CoreAuth exposes a comprehensive REST API. Key endpoint groups:

| Endpoint Group | Prefix | Description |
|----------------|--------|-------------|
| Health | `/health` | Health check |
| OAuth2/OIDC | `/authorize`, `/oauth/token`, `/userinfo` | Standard OAuth2 flows |
| Authentication | `/api/auth/*` | Register, login, refresh, logout |
| MFA | `/api/auth/mfa/*` | TOTP/SMS enrollment and verification |
| Passwordless | `/api/auth/passwordless/*` | Magic links and OTP |
| Tenant Management | `/api/tenants/*` | Users, invitations, security, branding |
| Applications | `/api/organizations/:id/applications` | OAuth2 client registration |
| SSO/OIDC Providers | `/api/oidc/providers` | External identity provider setup |
| FGA | `/api/fga/*` | Stores, models, tuples, permission checks |
| SCIM 2.0 | `/scim/v2/*` | User/group provisioning |
| Webhooks | `/api/organizations/:id/webhooks` | Event subscriptions |
| Audit Logs | `/api/organizations/:id/audit-logs` | Security event tracking |

Full route definitions: [`coreauth-core/crates/api/src/routes.rs`](coreauth-core/crates/api/src/routes.rs)

---

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
