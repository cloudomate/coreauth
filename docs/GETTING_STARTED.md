# Getting Started with CoreAuth

## Quick Start

### Prerequisites
- Docker and Docker Compose
- Git

### 1. Clone and Start

```bash
git clone https://github.com/cloudomate/coreauth.git
cd coreauth
./start.sh
```

This will start:
- PostgreSQL database (port 5432)
- Redis cache (port 6379)
- Backend API (port 8000)
- Frontend Dashboard (port 3000)

### 2. Access the Application

| Service | URL |
|---------|-----|
| Frontend Dashboard | http://localhost:3000 |
| Backend API | http://localhost:8000 |
| API Health Check | http://localhost:8000/health |

### 3. Create a Tenant

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

### 4. Login

```bash
curl -X POST http://localhost:8000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "tenant_id": "YOUR_TENANT_ID",
    "email": "admin@acme.com",
    "password": "SecureP@ssw0rd!"
  }'
```

### 5. Explore the Dashboard

Open http://localhost:3000 and log in with your admin credentials. From the dashboard you can manage:
- Users and groups
- OAuth2 applications
- Security policies (MFA, password rules, lockout)
- SSO/OIDC provider connections
- Branding (logo, colors, app name)
- Email templates
- Webhooks
- Audit logs

## Local Development

### Backend (Rust)

```bash
# Start infrastructure
docker compose up postgres redis -d

# Run backend
cd coreauth-core
cp .env.example .env
cargo run --bin coreauth-api
```

### Frontend (React)

```bash
cd coreauth-portal
npm install
npm run dev
```

### Sample App

```bash
cd samples/corerun-auth
npm install
npm run dev
```

## Environment Variables

Copy `.env.example` to `.env` in the project root:

```env
# Database
DATABASE_URL=postgresql://coreauth:change-this-in-production@postgres:5432/coreauth

# Redis
REDIS_URL=redis://redis:6379

# JWT (generate with: openssl rand -base64 64)
JWT_SECRET=your-secret-key-change-in-production

# Email (development)
EMAIL_PROVIDER=mailhog
MAILHOG_HOST=localhost
MAILHOG_PORT=1025

# Frontend
VITE_API_URL=http://localhost:8000
```

See `.env.example` for the complete list of configuration options.

## Docker Commands

```bash
# Start all services
docker compose up -d

# Rebuild and start
docker compose up --build -d

# View logs
docker compose logs -f

# Stop all services
docker compose down

# Stop and remove volumes (resets database)
docker compose down -v
```

## Next Steps

- [Developer Guide](DEVELOPER_GUIDE.md) - Full walkthrough from tenant creation to OAuth2 integration
- [Architecture](ARCHITECTURE.md) - System design and crate structure
- [SDK Integration](SDK_INTEGRATION.md) - Integrate CoreAuth into your application
- [Deployment](DEPLOYMENT.md) - Production deployment guide
