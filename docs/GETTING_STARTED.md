# Getting Started with CoreAuth

## Quick Start

### Prerequisites
- Docker and Docker Compose
- Git

### 1. Clone and Start

```bash
# Clone the repository
git clone <your-repo>
cd ciam

# Start all services with Docker Compose
./docker-start.sh
```

This will start:
- PostgreSQL database (port 5432)
- Redis cache (port 6379)
- Backend API (port 8000)
- Frontend UI (port 3000)

### 2. Access the Application

- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8000
- **API Health**: http://localhost:8000/health

### 3. First Login

1. Create an organization (tenant) via signup
2. Verify your email (check Docker logs for email content in dev mode)
3. Login with your credentials
4. If MFA is required, scan QR code with Google/Microsoft Authenticator

## Development Mode

For development with hot-reload:

```bash
# Frontend development
cd frontend
npm install
npm run dev

# Backend development
cd backend
cargo watch -x run
```

## Environment Variables

Create a `.env` file in the root directory:

```env
# Database
DATABASE_URL=postgresql://coreauth:coreauth@localhost:5432/coreauth

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key-change-in-production

# Email (for production)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
```

## Next Steps

- [Architecture Overview](ARCHITECTURE.md)
- [Authentication Guide](AUTHENTICATION.md)
- [Multi-Tenant Setup](MULTI_TENANT_ARCHITECTURE.md)
- [Deployment Guide](../DEPLOYMENT.md)
