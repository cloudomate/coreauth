# CoreAuth Portal

Professional, developer-centric authentication portal built with React and Tailwind CSS.

## Features

- 🎨 Modern, clean design that's better than Auth0
- 🚀 Lightning-fast React + Vite setup
- 🎯 Tailwind CSS for beautiful, professional styling
- 🔐 Complete authentication flow (signup, login, dashboard)
- 📱 Fully responsive design
- 🔄 Token refresh handling
- 🐳 Docker-ready

## Pages

1. **Landing Page** (`/`) - Marketing page with features and CTA
2. **Sign Up** (`/signup`) - Organization onboarding flow
3. **Login** (`/login`) - User authentication
4. **Dashboard** (`/dashboard`) - Protected dashboard with stats and quick actions

## Local Development

### Using Docker (Recommended)

```bash
# Build and start all services
docker compose -f docker-compose.test.yml up -d

# Access the portal
open http://localhost:3000
```

### Manual Setup

```bash
cd coreauth-portal

# Install dependencies
npm install

# Start dev server
npm run dev
```

## Project Structure

```
coreauth-portal/
├── public/              # Static assets
├── src/
│   ├── components/      # Reusable components
│   ├── pages/          # Page components
│   │   ├── Landing.jsx
│   │   ├── Signup.jsx
│   │   ├── Login.jsx
│   │   └── Dashboard.jsx
│   ├── lib/
│   │   └── api.js      # Axios API client
│   ├── App.jsx         # Main app component
│   ├── main.jsx        # Entry point
│   └── index.css       # Tailwind styles
├── index.html
├── vite.config.js
├── tailwind.config.js
└── Dockerfile
```

## API Integration

The portal connects to the CoreAuth API at `http://api:8000`. All API calls are proxied through nginx in production.

### Authentication Flow

1. User signs up via `/api/tenants` (creates organization + admin user)
2. Auto-login via `/api/auth/login-hierarchical`
3. Store access/refresh tokens in localStorage
4. Auto-refresh on 401 errors

## Design Philosophy

- **Developer-Centric**: Code examples, technical language, CLI-first aesthetic
- **Professional**: Clean, modern design without being overly playful
- **Better than Auth0**: Clearer information hierarchy, better onboarding flow
- **Performance**: Fast loading, optimized builds, efficient API calls

## Technologies

- **React 18** - UI framework
- **Vite** - Build tool
- **Tailwind CSS** - Styling
- **React Router** - Routing
- **Axios** - HTTP client
- **Docker** - Containerization
- **Nginx** - Production server

## Environment Variables

No environment variables needed! The portal uses relative API paths that are proxied in production.

## Building for Production

```bash
npm run build
```

Outputs to `dist/` directory.

## Deployment

The Docker image uses multi-stage builds:
1. **Build stage**: Node.js to build the React app
2. **Production stage**: Nginx to serve static files + proxy API

Access via `http://localhost:3000` when running with Docker Compose.
