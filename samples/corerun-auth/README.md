# CoreRun-Auth

A sample SaaS application ("CoreRun — Project Manager") that demonstrates integrating with **CoreAuth** as an identity provider.

## What it demonstrates

- **OAuth2/OIDC Login** — Server-side authorization code flow via CoreAuth Universal Login
- **Fine-Grained Authorization (FGA)** — Project-level permissions (owner/editor/viewer) via CoreAuth FGA API
- **User Management** — List and manage tenant users via CoreAuth APIs
- **SSO/MFA Configuration** — Configure OIDC providers and MFA policies for your tenant

## Prerequisites

- Node.js 18+
- CoreAuth running on `http://localhost:3000` (via `docker compose up`)
- An OAuth application registered in CoreAuth (webapp type)

## Quick Start

```bash
# Install dependencies
npm install

# Edit .env with your CoreAuth credentials
# (pre-configured for the default dev setup)

# Start the app
npm start
```

Visit `http://localhost:5050` and click "Sign in with CoreAuth".

## Configuration

Edit `.env`:

| Variable | Description | Default |
|----------|-------------|---------|
| `COREAUTH_BASE_URL` | CoreAuth API URL | `http://localhost:3000` |
| `COREAUTH_CLIENT_ID` | OAuth2 Client ID | (pre-configured) |
| `COREAUTH_CLIENT_SECRET` | OAuth2 Client Secret | (pre-configured) |
| `COREAUTH_CALLBACK_URL` | OAuth2 callback URL | `http://localhost:5050/oidc/callback` |
| `PORT` | App port | `5050` |
| `SESSION_SECRET` | Express session secret | (dev default) |
| `FGA_STORE_NAME` | FGA store name | `corerun-auth` |

## How FGA Works

On first login, the app creates an FGA store with this authorization model:

```
type user

type project
  relations
    define owner: [user]
    define editor: [user] or owner
    define viewer: [user] or editor
```

- **owner** → can view, edit, delete, and share the project
- **editor** → can view and edit (includes owners via inheritance)
- **viewer** → can view only (includes editors via inheritance)

When you create a project, an FGA tuple is written making you the owner.
When you share a project, additional tuples are written for the collaborator.
Every page access checks FGA permissions before rendering.

## Architecture

```
Browser → Express (port 5050)
            ├── /oidc/login     → redirect to CoreAuth /authorize
            ├── /oidc/callback  → exchange code → tokens → session
            ├── /dashboard      → project list (FGA-filtered)
            ├── /projects/:id   → FGA permission check
            ├── /admin/users    → CoreAuth user list API
            └── /admin/settings → CoreAuth security/SSO API
```

## Tech Stack

- **Node.js + Express** — Backend server
- **EJS** — Server-rendered templates (zero build step)
- **Tailwind CSS** — Styling via CDN
- **SQLite** — Local project storage
- **CoreAuth APIs** — OAuth2, FGA, user management, SSO/MFA
