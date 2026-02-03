# System Architecture

## Overview

CIAM follows a modern microservices-inspired architecture with clear separation of concerns between authentication, authorization, and identity management.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Client Layer                             │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐   │
│  │   Web App    │  │  Mobile App  │  │  Service Clients    │   │
│  │   (React)    │  │   (Native)   │  │  (API Integration)  │   │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬──────────┘   │
└─────────┼──────────────────┼───────────────────────┼─────────────┘
          │                  │                       │
          └──────────────────┼───────────────────────┘
                             │ HTTPS/REST
┌────────────────────────────┼─────────────────────────────────────┐
│                      API Gateway Layer                            │
│  ┌────────────────────────▼────────────────────────────────┐    │
│  │              Axum HTTP Server                            │    │
│  │  - CORS  - Rate Limiting  - Auth Middleware             │    │
│  └─────────────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────────────┘
                             │
┌────────────────────────────┼─────────────────────────────────────┐
│                     Service Layer                                 │
│  ┌──────────────┬──────────────┬────────────┬──────────────┐    │
│  │ Auth Service │ OIDC Service │ MFA Service│ Authz Engine │    │
│  ├──────────────┼──────────────┼────────────┼──────────────┤    │
│  │ Verification │   Password   │ Invitation │ Application  │    │
│  │   Service    │Reset Service │  Service   │   Service    │    │
│  └──────────────┴──────────────┴────────────┴──────────────┘    │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   Cache Layer (Redis)                    │    │
│  │  - Session Cache  - Rate Limit  - Auth Check Cache     │    │
│  └─────────────────────────────────────────────────────────┘    │
└────────────────────────────┬───────────────────────────────────┘
                             │
┌────────────────────────────┼───────────────────────────────────┐
│                     Data Layer                                   │
│  ┌─────────────────────────▼──────────────────────────────┐    │
│  │                   PostgreSQL                            │    │
│  │                                                          │    │
│  │  ┌──────────┐ ┌──────────┐ ┌────────────┐ ┌─────────┐ │    │
│  │  │  Tenants │ │  Users   │ │  Sessions  │ │  Roles  │ │    │
│  │  └──────────┘ └──────────┘ └────────────┘ └─────────┘ │    │
│  │  ┌──────────┐ ┌──────────┐ ┌────────────┐ ┌─────────┐ │    │
│  │  │   MFA    │ │   OIDC   │ │Invitations │ │ Tuples  │ │    │
│  │  └──────────┘ └──────────┘ └────────────┘ └─────────┘ │    │
│  │  ┌──────────┐                                           │    │
│  │  │   Apps   │                                           │    │
│  │  └──────────┘                                           │    │
│  └─────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
                             │
┌────────────────────────────┼───────────────────────────────────┐
│                  External Services                               │
│  ┌─────────────────┐  ┌──────────────────┐  ┌──────────────┐  │
│  │  MailHog SMTP   │  │  SMPP Gateway    │  │ OIDC Providers│ │
│  │ (Email Gateway) │  │  (SMS Gateway)   │  │ (Auth0, etc.) │ │
│  └─────────────────┘  └──────────────────┘  └───────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. API Layer

**Technology:** Axum (Rust async web framework)

The API layer handles all HTTP requests and provides:
- REST endpoints for all operations
- Request validation
- Authentication middleware
- Rate limiting
- CORS configuration

### 2. Authentication Services

#### Auth Service
- User registration and login
- Password hashing (Argon2)
- JWT token generation and validation
- Session management
- Password reset flows

#### OIDC Service
- OpenID Connect integration
- Support for Auth0, Google Workspace, Azure AD
- Group synchronization
- JWT token parsing
- Template-based provider configuration

#### MFA Service
- TOTP enrollment and verification
- QR code generation
- Backup code management
- MFA enforcement policies

### 3. Authorization Services

#### Policy Engine
- Zanzibar-style permission checking
- ReBAC (Relationship-Based Access Control)
- Graph traversal for permission resolution
- Caching for performance
- Forward auth for downstream apps

#### Application Service
- Service principal management
- Client credentials generation
- Application authentication
- Secret rotation

#### Tuple Service
- Relation tuple CRUD operations
- Subject-object-relation storage
- Query capabilities

### 4. Data Layer

#### PostgreSQL Database
- Multi-tenant data isolation
- ACID transactions
- Full-text search capabilities
- JSON support for flexible schemas

**Schema Overview:**
- `tenants` - Tenant configuration
- `users` - User accounts
- `sessions` - Active sessions
- `roles` - Role definitions
- `user_roles` - Role assignments
- `mfa_methods` - MFA enrollments
- `oidc_providers` - OIDC configuration
- `applications` - Service principals
- `relation_tuples` - Authorization tuples
- `invitations` - User invitations

#### Redis Cache
- Session storage
- Rate limiting counters
- Authorization check cache
- OIDC state storage

### 5. External Services

#### Email Gateway (MailHog)
- Host: datacore.lan:1025
- Web UI: https://mailhog.imys.in
- Protocol: SMTP
- Usage: Email verification, password reset, invitations

#### SMS Gateway (SMPP)
- Host: datacore.lan:2775 (or sms.imys.in)
- Protocol: SMPP 3.4
- Usage: OTP delivery, notifications

## Multi-Tenancy Architecture

### Tenant Isolation Models

1. **Pool Model** (Default)
   - Shared database with tenant_id filtering
   - Most cost-effective
   - Suitable for most use cases

2. **Silo Model**
   - Separate database per tenant
   - Maximum isolation
   - For enterprise customers

### Tenant Context

All requests include tenant context through:
- Tenant slug in URL
- Tenant ID in JWT claims
- Middleware enforcement

## Security Architecture

### Authentication Flow

```
1. User submits credentials
   ↓
2. Server validates credentials
   ↓
3. Check MFA requirement
   ↓
4. Generate JWT tokens (access + refresh)
   ↓
5. Store session in Redis
   ↓
6. Return tokens to client
```

### Authorization Flow (ReBAC)

```
1. Request with JWT token
   ↓
2. Extract subject from token
   ↓
3. Build permission check request
   ↓
4. Query relation tuples
   ↓
5. Perform graph traversal
   ↓
6. Check cache for result
   ↓
7. Return allow/deny decision
```

### Password Security

- **Algorithm:** Argon2id
- **Salt:** Unique per password
- **Iterations:** Tuned for security/performance
- **Memory:** Configured for DoS resistance

### Token Security

- **Access Token:** Short-lived (1 hour)
- **Refresh Token:** Long-lived (30 days)
- **Algorithm:** RS256 (asymmetric)
- **Claims:** User ID, tenant ID, roles

## Scalability Considerations

### Horizontal Scaling

- **Stateless API servers** - Can run multiple instances
- **Load balancing** - Distribute across instances
- **Redis cluster** - For distributed caching
- **PostgreSQL replication** - Read replicas for queries

### Performance Optimizations

1. **Caching Strategy**
   - Authorization checks cached in Redis (60s TTL)
   - Session data cached
   - Rate limit counters in memory

2. **Database Optimization**
   - Indexed foreign keys
   - Composite indexes on query patterns
   - Connection pooling (SQLx)

3. **Async Processing**
   - Email sending in background tasks
   - Non-blocking I/O throughout
   - Tokio runtime for concurrency

## Monitoring & Observability

### Logging

- **Framework:** tracing (Rust)
- **Format:** JSON structured logs
- **Levels:** DEBUG, INFO, WARN, ERROR
- **Correlation:** Request IDs

### Metrics

- Request latency
- Error rates
- Cache hit rates
- Database query performance

## Deployment Architecture

### Container Structure

```yaml
services:
  frontend:    # React application (port 3000)
  backend:     # Rust API server (port 8003)
  postgres:    # PostgreSQL database (port 5434)
  redis:       # Redis cache (port 6379)
  pgadmin:     # Database admin UI (port 5050)
  redis-commander: # Redis admin UI (port 8082)
```

### Environment Configuration

- Development: Docker Compose
- Production: Kubernetes (recommended)
- External services: Configured via environment variables

## API Design Principles

1. **RESTful** - Standard HTTP methods and status codes
2. **Versioned** - API version in URL path
3. **Consistent** - Uniform response formats
4. **Documented** - OpenAPI/Swagger compatible
5. **Secure** - Authentication required by default
6. **Idempotent** - Safe retry behavior

## Data Flow Examples

### User Registration Flow

```
Frontend → POST /api/auth/register
         → Backend validates input
         → Hash password (Argon2)
         → Store user in PostgreSQL
         → Send verification email (background)
         → Return JWT tokens
```

### OIDC Login Flow

```
Frontend → GET /api/oidc/login?provider_id=xyz
         → Backend generates OAuth state
         → Store state in Redis
         → Redirect to OIDC provider
         → User authenticates
         → Provider redirects to callback
         → Backend validates state & token
         → Extract user info & groups
         → Sync groups to roles
         → Create/update user
         → Return JWT tokens
```

### Permission Check Flow

```
Downstream App → POST /authz/forward-auth
               → Backend extracts subject
               → Query relation_tuples
               → Perform graph traversal
               → Check cache
               → Return 200 (allow) or 403 (deny)
```

## Future Considerations

- **Event Sourcing** - Audit log of all changes
- **CQRS** - Separate read/write models
- **GraphQL** - Alternative API interface
- **gRPC** - For service-to-service communication
- **Kubernetes** - Container orchestration
- **Service Mesh** - Advanced networking (Istio/Linkerd)
