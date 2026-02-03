# CoreAuth - Enterprise CIAM Platform

Multi-tenant Customer Identity and Access Management (CIAM) platform built with Rust and React.

## Features

- 🔐 **Multi-Tenant Architecture** - Complete tenant isolation with hierarchical organizations
- 🛡️ **Multi-Factor Authentication** - TOTP support (Google/Microsoft Authenticator)
- 👥 **User Management** - Comprehensive user lifecycle management
- 🔑 **SSO/OIDC** - Enterprise SSO integration
- 📊 **Fine-Grained Authorization** - Tuple-based permissions (Zanzibar-inspired)
- 📧 **Email/SMS** - Built-in notification system
- 🎯 **Actions/Hooks** - JavaScript-based extensibility
- 📝 **Audit Logging** - Complete audit trail
- 🚀 **High Performance** - Built with Rust for speed and reliability

## Quick Start

```bash
# Start all services with Docker Compose
./docker-start.sh

# Access the application
open http://localhost:3000
```

**Default Services:**
- Frontend: http://localhost:3000
- Backend API: http://localhost:8000
- API Health: http://localhost:8000/health

## Architecture

### Technology Stack

**Backend:**
- Rust with Axum web framework
- PostgreSQL for persistent storage
- Redis for caching and sessions
- SQLx for type-safe database queries

**Frontend:**
- React 18 with Vite
- React Router v6
- TailwindCSS
- Axios for API calls

### Multi-Tenant Model

```
Platform (CoreAuth)
├── Organizations (Tenants)
│   ├── Users & Roles
│   ├── Applications (OAuth2 clients)
│   ├── Connections (SSO providers)
│   ├── Security Settings
│   └── Child Organizations (Hierarchical)
└── Platform Admins
```

## Project Structure

```
ciam/
├── backend/                 # Rust backend
│   ├── crates/
│   │   ├── api/            # API handlers & routes
│   │   ├── auth/           # Authentication & MFA logic
│   │   ├── cache/          # Redis cache abstraction
│   │   ├── database/       # Database repositories
│   │   └── models/         # Domain models
│   ├── migrations/         # SQL migrations
│   └── Cargo.toml
├── frontend/               # React frontend
│   ├── src/
│   │   ├── components/    # Reusable UI components
│   │   ├── pages/         # Page components
│   │   └── lib/           # Utilities & API client
│   └── package.json
├── docs/                   # Documentation
│   ├── GETTING_STARTED.md
│   ├── ARCHITECTURE.md
│   ├── AUTHENTICATION.md
│   ├── AUTHORIZATION.md
│   ├── DEPLOYMENT.md
│   └── TESTING.md
└── docker-compose.yml
```

## Key Capabilities

### Authentication
- Email/password login with bcrypt hashing
- Multi-factor authentication (TOTP)
- Enrollment token-based MFA setup
- JWT access and refresh tokens
- Session management with Redis

### Multi-Tenancy
- Complete tenant data isolation
- Per-tenant security policies
- Hierarchical organization support
- Tenant-scoped applications and connections

### Authorization
- Role-based access control (RBAC)
- Tuple-based fine-grained permissions
- Permission inheritance in hierarchies
- Forward auth for downstream services

### Security Features
- MFA enforcement at organization level
- Password policy configuration
- Grace periods for MFA enrollment
- Backup codes for account recovery
- Comprehensive audit logging

## Documentation

- **[Getting Started](docs/GETTING_STARTED.md)** - Setup and first steps
- **[Architecture](docs/ARCHITECTURE.md)** - System design and components
- **[Authentication](docs/AUTHENTICATION.md)** - Auth flows and MFA
- **[Authorization](docs/AUTHORIZATION.md)** - Permissions and RBAC
- **[Multi-Tenant Architecture](docs/MULTI_TENANT_ARCHITECTURE.md)** - Tenant isolation
- **[Deployment](docs/DEPLOYMENT.md)** - Production deployment
- **[Testing](docs/TESTING.md)** - Testing guide
- **[Email/SMS Setup](docs/EMAIL_SMS_SETUP.md)** - Notification configuration

## API Endpoints

### Authentication
```
POST   /api/auth/register              # User registration
POST   /api/auth/login-hierarchical    # Tenant-scoped login
POST   /api/auth/refresh               # Refresh access token
POST   /api/auth/logout                # Logout
GET    /api/auth/me                    # Get current user
```

### MFA
```
POST   /api/mfa/enroll-with-token/totp        # Start TOTP enrollment
POST   /api/mfa/verify-with-token/totp/:id    # Verify and complete enrollment
GET    /api/mfa/methods                        # List user's MFA methods
POST   /api/mfa/backup-codes/regenerate        # Generate new backup codes
```

### Organizations
```
POST   /api/tenants                    # Create organization
GET    /api/tenants/:id/users          # List organization users
PUT    /api/tenants/:id/users/:user_id/role  # Update user role
GET    /api/organizations/:id/security # Get security settings
PUT    /api/organizations/:id/security # Update security settings
```

See [API Reference](docs/api/) for complete endpoint documentation.

## Development

### Prerequisites
- Docker & Docker Compose
- Rust 1.70+ (for local development)
- Node.js 18+ (for local development)

### Local Development

```bash
# Backend
cd backend
cargo watch -x run

# Frontend
cd frontend
npm install
npm run dev

# Database
docker compose up -d postgres redis
```

### Running Tests

```bash
# Backend tests
cd backend
cargo test

# Integration tests
cargo test --test integration_tests

# Frontend tests
cd frontend
npm test
```

See [Testing Guide](docs/TESTING.md) for comprehensive testing documentation.

## Environment Variables

Create `.env` file in the root directory:

```env
# Database
DATABASE_URL=postgresql://coreauth:coreauth@localhost:5432/coreauth

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key-here

# Application
RUST_LOG=info
FRONTEND_URL=http://localhost:3000

# Email
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
```

## Deployment

### Docker Compose (Recommended)

```bash
# Production deployment
docker compose -f docker-compose.prod.yml up -d
```

### Cloud Platforms

- AWS (ECS + RDS)
- Google Cloud (Cloud Run + Cloud SQL)
- DigitalOcean App Platform
- Kubernetes (Helm charts available)

See [Deployment Guide](docs/DEPLOYMENT.md) for detailed instructions.

## Roadmap

### Completed ✅
- Multi-tenant architecture
- JWT authentication
- MFA with TOTP
- Role-based access control
- Security settings UI
- Audit logging
- Tuple-based authorization
- Email notifications

### In Progress 🚧
- Actions/Hooks system (JavaScript extensibility)
- OAuth2 provider capabilities
- WebAuthn support
- Advanced analytics dashboard

### Planned 📋
- SAML 2.0 support
- Social login providers
- Passwordless authentication
- Multi-region deployment
- Admin API for tenant management

## Contributing

We welcome contributions! Please see our contributing guidelines.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

[Your License Here]

## Support

- Documentation: [/docs](docs/)
- Issues: [GitHub Issues](https://github.com/your-org/coreauth/issues)
- Email: support@coreauth.dev

## Security

For security issues, please email security@coreauth.dev instead of using the issue tracker.

---

Built with ❤️ using Rust and React
