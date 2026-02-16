# CoreAuth On-Prem Deployment

Single-tenant, headless identity API packaged for on-prem deployment.
CoreAuth runs as a single Docker container alongside your application.
You provide your own PostgreSQL and Redis.

## Prerequisites

- Docker Engine 20+ with Compose v2
- PostgreSQL 14+
- Redis 6+
- `openssl` (for secret generation)

## Quick Start

```bash
cd deploy/onprem
./setup.sh
```

This will:
1. Generate `.env` with secrets (JWT key, admin password)
2. Prompt you to configure database and Redis connection
3. Validate connectivity to PostgreSQL and Redis
4. Start the CoreAuth backend container
5. Auto-provision your tenant, admin user, and OAuth application
6. Print your credentials (client_id, client_secret)

## With Admin Dashboard

```bash
./setup.sh --with-dashboard
```

Adds the admin UI on port 8080 (configurable via `FRONTEND_PORT` in `.env`).

## Configuration

All configuration is in `.env` (created from `.env.onprem.template` on first run).

### Required

| Variable | Description |
|---|---|
| `POSTGRES_HOST` | PostgreSQL hostname |
| `POSTGRES_PORT` | PostgreSQL port (default: 5432) |
| `POSTGRES_USER` | Database user |
| `POSTGRES_PASSWORD` | Database password |
| `POSTGRES_DB` | Database name (default: coreauth) |
| `REDIS_URL` | Redis connection URL |
| `TENANT_NAME` | Your organization name |
| `TENANT_SLUG` | URL-safe identifier (e.g., `my-org`) |
| `ADMIN_EMAIL` | Admin user email |
| `APP_CALLBACK_URLS` | OAuth callback URLs (comma-separated) |

### Auto-generated

| Variable | Description |
|---|---|
| `JWT_SECRET` | 64-byte signing key |
| `ADMIN_PASSWORD` | Initial admin password |

### Optional

| Variable | Default | Description |
|---|---|---|
| `BACKEND_PORT` | 3000 | API listen port |
| `FRONTEND_PORT` | 8080 | Dashboard port (with `--with-dashboard`) |
| `EMAIL_PROVIDER` | console | `console`, `smtp` |
| `CORS_ORIGINS` | * | Allowed CORS origins |
| `REQUIRE_EMAIL_VERIFICATION` | false | Require email verification |
| `SMS_ENABLED` | false | Enable SMS MFA |

## Integration

After setup completes, you'll receive:

- **API URL**: `http://localhost:3000` (or your configured port)
- **Client ID**: Your OAuth application identifier
- **Client Secret**: Your OAuth application secret
- **Admin Email/Password**: Dashboard login credentials

### Authentication Flow

```bash
# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "my-org",
    "email": "user@example.com",
    "password": "password"
  }'

# Response: { "access_token": "...", "refresh_token": "...", "user": {...} }
```

### OAuth2 / OIDC

```bash
# OpenID Configuration
curl http://localhost:3000/.well-known/openid-configuration

# Token endpoint (client credentials)
curl -X POST http://localhost:3000/oauth/token \
  -d "grant_type=client_credentials&client_id=CLIENT_ID&client_secret=CLIENT_SECRET"
```

### Key API Endpoints

| Endpoint | Description |
|---|---|
| `GET /health` | Health check |
| `POST /api/auth/login` | User login |
| `POST /api/auth/register` | User registration |
| `GET /api/auth/me` | Current user profile |
| `POST /api/auth/refresh` | Refresh token |
| `POST /oauth/token` | OAuth2 token endpoint |
| `GET /userinfo` | OIDC userinfo |
| `GET /.well-known/openid-configuration` | OIDC discovery |

## Operations

### Logs

```bash
docker logs coreauth-core
docker logs coreauth-bootstrap
```

### Stop

```bash
docker compose -f docker-compose.onprem.yml down
```

### Restart

```bash
docker compose -f docker-compose.onprem.yml restart backend
```

### Re-bootstrap

Remove the sentinel file and restart the bootstrap container:

```bash
docker volume rm coreauth_bootstrap
docker compose -f docker-compose.onprem.yml up bootstrap
```

### Update

```bash
docker compose -f docker-compose.onprem.yml build --pull
docker compose -f docker-compose.onprem.yml up -d
```

## Architecture

```
Your Infrastructure
├── PostgreSQL (you manage)
├── Redis (you manage)
├── CoreAuth API (this package)
│   └── Runs migrations on startup
│   └── Exposes REST API + OAuth2/OIDC
├── CoreAuth Dashboard (optional, --with-dashboard)
└── Your Application
    └── Integrates via OAuth2 / API
```

## Credentials File

Bootstrap writes credentials to a Docker volume at `/data/credentials.json`:

```json
{
  "tenant_id": "uuid",
  "tenant_slug": "my-org",
  "admin_email": "admin@example.com",
  "client_id": "uuid",
  "client_secret": "secret",
  "api_url": "http://backend:3000"
}
```

Access it:
```bash
docker run --rm -v coreauth_bootstrap:/data alpine cat /data/credentials.json
```
