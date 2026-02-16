# CIAM - Customer Identity & Access Management

## Overview

CIAM is a comprehensive, enterprise-grade Customer Identity and Access Management system built with Rust and React. It provides multi-tenant authentication, authorization, and identity management capabilities.

## Key Features

- **Multi-Tenant Architecture** - Isolated tenant data with flexible isolation models
- **Multiple Authentication Methods** - Passwords, OIDC/OAuth2, MFA (TOTP)
- **Fine-Grained Authorization** - Zanzibar-style ReBAC and ABAC
- **Service Principals** - Application identity management
- **Email & SMS** - Configurable providers for notifications
- **Security** - Argon2 password hashing, JWT tokens, rate limiting
- **Scalability** - Redis caching, PostgreSQL, async Rust backend

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (React)                         │
│  - Admin Dashboard  - User Profile  - OIDC Login             │
└──────────────────────┬──────────────────────────────────────┘
                       │ REST API
┌──────────────────────┴──────────────────────────────────────┐
│              Backend API (Rust/Axum)                         │
│  ┌─────────────┬──────────────┬────────────┬──────────────┐ │
│  │   Auth      │    OIDC      │    MFA     │   Authz      │ │
│  │  Service    │   Service    │  Service   │   Engine     │ │
│  └─────────────┴──────────────┴────────────┴──────────────┘ │
└──────────────────────┬──────────────────────────────────────┘
                       │
        ┌──────────────┴────────────────┐
        │                               │
┌───────▼────────┐              ┌───────▼────────┐
│   PostgreSQL   │              │     Redis      │
│   (Database)   │              │    (Cache)     │
└────────────────┘              └────────────────┘
```

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Rust 1.70+ (for local development)
- Node.js 18+ (for frontend development)
- PostgreSQL 16
- Redis 7

### Running with Docker Compose

```bash
docker-compose up -d
```

Services:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8003
- PostgreSQL: localhost:5434
- Redis: localhost:6379
- PgAdmin: http://localhost:5050

### Local Development

1. **Start infrastructure:**
   ```bash
   docker-compose up postgres redis -d
   ```

2. **Run migrations:**
   ```bash
   cd backend
   sqlx migrate run
   ```

3. **Start backend:**
   ```bash
   cd backend
   cargo run --bin ciam-api
   ```

4. **Start frontend:**
   ```bash
   cd coreauth-portal
   npm install
   npm start
   ```

## Documentation

- [Architecture](./ARCHITECTURE.md) - System architecture and design
- [Features](./FEATURES.md) - Complete feature list with details
- [Authentication](./AUTHENTICATION.md) - Authentication methods and flows
- [Authorization](./AUTHORIZATION.md) - ReBAC/ABAC and permission model
- [Configuration](./CONFIGURATION.md) - Configuration guide
- [API Reference](./API.md) - REST API documentation

## Technology Stack

### Backend
- **Language:** Rust
- **Framework:** Axum (Web), Tokio (Async Runtime)
- **Database:** PostgreSQL 16 with SQLx
- **Cache:** Redis 7
- **Authentication:** Argon2, JWT, OIDC
- **Authorization:** Custom Zanzibar-style engine

### Frontend
- **Framework:** React 18 with TypeScript
- **UI Library:** Material-UI (MUI)
- **State Management:** React Query
- **Routing:** React Router

### Infrastructure
- **Containerization:** Docker
- **Orchestration:** Docker Compose
- **Email Gateway:** External MailHog (SMTP)
- **SMS Gateway:** External SMPP Gateway

## Security Features

- ✅ Argon2id password hashing
- ✅ JWT access & refresh tokens
- ✅ TOTP-based MFA
- ✅ Rate limiting on sensitive endpoints
- ✅ Email verification
- ✅ Password reset flows
- ✅ Session management
- ✅ Tenant isolation
- ✅ OIDC group synchronization
- ✅ Client credentials for service principals

## License

Apache-2.0

## Support

For issues and questions, please refer to the documentation or contact the development team.
