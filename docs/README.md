# CoreAuth Documentation

## Overview

CoreAuth is a multi-tenant Customer Identity and Access Management (CIAM) platform. It provides authentication, authorization, and identity management for SaaS applications.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                  CoreAuth Proxy (:4000)                       │
│          OAuth2 sessions, X-CoreAuth-* header injection       │
└────────┬────────────────────────────┬────────────────────────┘
         │                            │
┌────────▼──────────────┐   ┌────────▼──────────────┐
│   Backend API          │   │   Frontend Dashboard   │
│   (Rust/Axum)          │   │   (React/Vite)         │
│   :3000 int / :8000 ext│   │   :3000 (nginx)        │
│                        │   │                        │
│  ┌──────┬──────┬─────┐ │   │  Admin UI for:         │
│  │ Auth │ OIDC │ MFA │ │   │  - Users & groups      │
│  │ FGA  │ SCIM │Audit│ │   │  - Applications        │
│  └──────┴──────┴─────┘ │   │  - Security settings   │
└────────┬───────┬───────┘   │  - SSO & branding      │
         │       │           └────────────────────────┘
  ┌──────▼──┐ ┌──▼─────┐
  │PostgreSQL│ │ Redis  │
  │  :5432   │ │ :6379  │
  └─────────┘ └────────┘
```

## Documentation Index

### Getting Started
- [Getting Started](GETTING_STARTED.md) - Quick start with Docker
- [Developer Guide](DEVELOPER_GUIDE.md) - Full walkthrough: tenant setup through OAuth2 integration

### Architecture & Design
- [Architecture](ARCHITECTURE.md) - System architecture, crate structure, data flow
- [Multi-Tenant Architecture](MULTI_TENANT_ARCHITECTURE.md) - Tenant isolation, database routing
- [Features](FEATURES.md) - Complete feature list

### Authentication & Authorization
- [Authentication](AUTHENTICATION.md) - Auth methods, flows, JWT, MFA
- [Authorization](AUTHORIZATION.md) - FGA engine, ReBAC, tuple management

### Integration
- [SDK Integration](SDK_INTEGRATION.md) - Node.js, Python, Go SDK usage
- [Email & SMS Setup](EMAIL_SMS_SETUP.md) - Provider configuration

### Operations
- [Deployment](DEPLOYMENT.md) - Docker, Kubernetes, cloud deployment
- [Testing](TESTING.md) - Running tests

## Technology Stack

### Backend (`coreauth-core/`)
| Component | Technology |
|-----------|-----------|
| Language | Rust (2021 edition) |
| Web Framework | Axum 0.7, Tokio |
| Database | PostgreSQL 16 with SQLx 0.8 |
| Cache | Redis 7 |
| Auth | JWT (RS256 + HS256), Argon2id, TOTP |
| Authorization | Zanzibar-style FGA engine |
| Scripting | Deno Core (sandboxed action hooks) |

### Frontend (`coreauth-portal/`)
| Component | Technology |
|-----------|-----------|
| Framework | React 18 |
| Build Tool | Vite 5 |
| Styling | Tailwind CSS 3.4 |
| Routing | React Router 6 |
| HTTP Client | Axios |

### Proxy (`coreauth-proxy/`)
| Component | Technology |
|-----------|-----------|
| Language | Rust |
| Framework | Axum 0.7, Hyper 1 |
| Features | Reverse proxy, JWT validation, session management, FGA integration |

### SDKs (`sdk/`)
| SDK | Language | Package |
|-----|----------|---------|
| Node.js | TypeScript | `@coreauth/sdk` |
| Python | Python 3.9+ | `coreauth` |
| Go | Go | `github.com/cloudomate/coreauth/sdk/go` |

## Security Features

- Argon2id password hashing
- JWT access and refresh tokens (RS256 + HS256)
- TOTP and SMS-based MFA with backup codes
- Passwordless authentication (magic links, OTP)
- Rate limiting on authentication endpoints
- Email verification
- Password reset with signed tokens
- Session management and revocation
- Tenant data isolation
- AES-GCM encryption for sensitive database fields
- SCIM 2.0 provisioning
- Audit logging
- Account lockout after failed attempts

## Ports

| Service | Port | Description |
|---------|------|-------------|
| Proxy | 4000 | Entry point (when using proxy) |
| Backend API (internal) | 3000 | Direct backend access |
| Backend API (external) | 8000 | Nginx-proxied backend |
| Frontend | 3000 | React dashboard (nginx) |
| Sample App | 3001 | Demo SaaS application |
| PostgreSQL | 5432 | Database |
| Redis | 6379 | Cache |

## License

Apache-2.0. See [LICENSE](../LICENSE).
