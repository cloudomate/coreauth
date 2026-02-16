# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

CoreAuth — a multi-tenant CIAM (Customer Identity and Access Management) platform. Rust backend (Axum), React frontend (Vite), Rust reverse proxy, PostgreSQL, Redis. Includes a sample SaaS app (`samples/corerun-auth/`) demonstrating FGA integration.

## Build & Run Commands

### Full Stack (Docker)
```bash
docker compose up --build          # Build and start everything
docker compose up --build -d       # Detached mode
docker compose down -v             # Tear down including volumes (resets DB)
docker compose build --no-cache backend  # Rebuild single service
```

### Backend (Rust — `coreauth-core/`)
```bash
cd coreauth-core
cargo check                              # Fast compilation check (preferred over cargo build)
SQLX_OFFLINE=true cargo check            # Required when DB is not available (uses .sqlx cache)
cargo build --release --locked           # Production build
cargo test                               # Run tests
cargo test test_name -- --nocapture      # Single test with output
```

**Important:** The backend has ~80 pre-existing warnings in `cargo check` — this is normal. SQLx offline mode uses cached query metadata in `coreauth-core/.sqlx/` and `coreauth-core/crates/database/.sqlx/` directories. When modifying SQL queries, the `.sqlx` cache files need updating (run against a live DB without `SQLX_OFFLINE`).

### Frontend (React)
```bash
cd coreauth-portal
npm install
npm run dev       # Dev server on port 3000
npm run build     # Production build
```

### Sample App (Express.js)
```bash
cd samples/corerun-auth
npm install
npm run dev       # Dev server with watch mode on port 3001
npm start         # Production start
```

### Proxy (Rust)
```bash
cd coreauth-proxy
cargo build --release
# Binary: coreauth-proxy, default config: proxy.yaml, default port: 4000
```

## Architecture

### Service Topology
```
Browser → Proxy (:4000) → Backend API (:3000/8000)
                        → Sample App (:3001)
                        → Frontend (:3000, nginx)
         PostgreSQL (:5432), Redis (:6379)
```

The proxy handles OAuth2 session management and injects `X-CoreAuth-*` headers (User-Id, User-Email, Tenant-Id, Token, Role, etc.) into upstream requests. Protected apps never see raw credentials — they read identity from these headers.

### Backend Crate Dependency Graph
```
api → auth, authz, database, models, tenant, cache
auth → models, database, cache
authz → models, database, cache
tenant → models, database, cache
database → models
models → (standalone, serde + sqlx + validator)
cache → (standalone, redis)
```

- **api**: Axum handlers, routes, AppState initialization, middleware. Binary: `coreauth-api`.
- **auth**: Authentication flows (login, signup, MFA/TOTP, passwordless, OAuth2/OIDC, social login, email verification, password reset, JWT with RS256+HS256), email/SMS delivery.
- **authz**: Fine-Grained Authorization — PolicyEngine (model-aware Zanzibar-style), FGA stores, tuple management, authorization model schema.
- **database**: SQLx repositories for all entities, tenant database router, AES-GCM encryption for sensitive fields.
- **models**: Domain structs with serde + sqlx + validator derives. Shared across all crates.
- **tenant**: Multi-tenant isolation, tenant registry, tenant database routing.
- **cache**: Redis abstraction layer.

### FGA (Fine-Grained Authorization) System
The authorization engine in `coreauth-core/crates/authz/` is OpenFGA-compatible (Zanzibar-style ReBAC):

- **PolicyEngine** (`engine.rs`): Model-aware permission resolution. Resolves `computedUserset` (role hierarchy like owner→editor→viewer), `tupleToUserset` (cross-type inheritance like workspace admin → resource owner), and union/intersection/exclusion operations.
- **store.rs**: `AuthorizationSchema`, `TypeDefinition`, `RelationDefinition` with serde aliases for both camelCase and snake_case.
- **Serde gotchas**: `TypeDefinition.relations` and `DirectRelation.types` need `#[serde(default)]`. Union/intersection fields use a custom deserializer to handle both `[...]` and `{ child: [...] }` formats.
- The `WriteModelRequest` expects `{ schema: AUTH_MODEL }`, not the model directly.

### Sample App (`samples/corerun-auth/`)
Cloud infrastructure console demonstrating deep FGA with 14 resource types (workspaces, compute instances/functions, storage buckets/volumes/databases, network VPCs/subnets/firewalls/load balancers).

- `services/fga.js`: AUTH_MODEL definition + FGA API helpers
- `routes/resources.js`: Data-driven CRUD via `RESOURCE_TYPES` config map (9 types with type-specific fields, actions, roles, statuses)
- `routes/workspaces.js`: Workspace CRUD with FGA admin/member/viewer
- `middleware/fga.js`: `requirePermission(objectType, relation, paramName)` Express middleware
- `bootstrap.sh`: Init container that creates tenant, admin user, OAuth app, proxy config, email templates

### Database Migrations
Located in `coreauth-core/migrations/` (001–013). Run sequentially by `docker-entrypoint.sh` on startup using `psql`. Not managed by a migration framework — just raw SQL files executed in order.

### Proxy Configuration
The proxy reads `proxy.yaml` (generated by bootstrap.sh) defining route matching rules with auth modes: `none`, `optional`, `required`. Routes can target `coreauth` (backend) or upstream (sample app). The proxy manages FGA store initialization per-app via the `fga.store_name` config.

## Key Conventions

- Backend uses `SQLX_OFFLINE=true` in Docker builds; `.sqlx/` cache directories must be committed
- Proxy injects identity via `X-CoreAuth-*` headers — downstream apps should never handle raw auth
- Multi-tenancy: tenant_id is extracted from JWT and threaded through all service calls
- Sample app uses SQLite locally (better-sqlite3), not PostgreSQL
- EJS templates in `samples/corerun-auth/views/` use Tailwind via CDN (`cdn.tailwindcss.com`)
- Frontend uses Tailwind with a custom brand color palette defined in `tailwind.config.js`

## Ports
| Service | Port |
|---------|------|
| Proxy (entry point) | 4000 |
| Backend API (internal) | 3000 |
| Backend API (external/nginx) | 8000 |
| Frontend (nginx) | 3000 |
| Sample App | 3001 |
| PostgreSQL | 5432 |
| Redis | 6379 |
