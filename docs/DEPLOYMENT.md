# Deployment Guide

## Production Deployment

### Using Docker Compose (Recommended)

#### 1. Prepare Environment

```bash
cp .env.example .env
nano .env
```

Required environment variables:

```env
# Database
DATABASE_URL=postgresql://coreauth:STRONG_PASSWORD@postgres:5432/coreauth

# Redis
REDIS_URL=redis://redis:6379

# JWT Secret (generate with: openssl rand -base64 64)
JWT_SECRET=your-very-long-secret-key-change-this

# Application
RUST_LOG=info
FRONTEND_URL=https://your-domain.com
APP_URL=https://api.your-domain.com

# Email (Production SMTP)
EMAIL_PROVIDER=smtp
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
EMAIL_FROM=noreply@your-domain.com

# SMS (Optional)
SMS_ENABLED=false
# SMS_PROVIDER=twilio
# TWILIO_ACCOUNT_SID=your-account-sid
# TWILIO_AUTH_TOKEN=your-auth-token
# TWILIO_FROM_NUMBER=+1234567890

# CORS
CORS_ORIGINS=https://your-domain.com
```

#### 2. Deploy

```bash
docker compose up --build -d
docker compose ps
docker compose logs -f
```

#### 3. Reverse Proxy (Nginx)

```nginx
# /etc/nginx/sites-available/coreauth

upstream backend {
    server localhost:3000;
}

upstream frontend {
    server localhost:3000;
}

# API Server
server {
    listen 443 ssl http2;
    server_name api.your-domain.com;

    ssl_certificate /etc/letsencrypt/live/api.your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.your-domain.com/privkey.pem;

    location / {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}

# Frontend Server
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    location / {
        proxy_pass http://frontend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name api.your-domain.com your-domain.com;
    return 301 https://$server_name$request_uri;
}
```

```bash
ln -s /etc/nginx/sites-available/coreauth /etc/nginx/sites-enabled/
nginx -t
systemctl restart nginx
```

### Cloud Deployment

#### AWS (ECS + RDS)

1. **RDS PostgreSQL** - Engine: PostgreSQL 16, Instance: db.t3.medium+, Storage: 50GB+ gp3
2. **ElastiCache Redis** - Engine: Redis 7, Node type: cache.t3.micro+
3. **Deploy to ECS:**

```bash
docker build -t your-registry/coreauth-core:latest coreauth-core/
docker build -t your-registry/coreauth-ui:latest coreauth-portal/
docker push your-registry/coreauth-core:latest
docker push your-registry/coreauth-ui:latest
```

#### Google Cloud (Cloud Run + Cloud SQL)

```bash
# Backend
gcloud builds submit --tag gcr.io/PROJECT_ID/coreauth-core coreauth-core/
gcloud run deploy coreauth-core \
  --image gcr.io/PROJECT_ID/coreauth-core \
  --platform managed \
  --region us-central1 \
  --add-cloudsql-instances PROJECT_ID:us-central1:coreauth-db

# Frontend
gcloud builds submit --tag gcr.io/PROJECT_ID/coreauth-ui coreauth-portal/
gcloud run deploy coreauth-ui \
  --image gcr.io/PROJECT_ID/coreauth-ui \
  --platform managed \
  --region us-central1
```

### Kubernetes (Helm)

Helm charts are in `deploy/helm/coreauth/`.

```bash
helm install coreauth deploy/helm/coreauth/ \
  --set postgresql.enabled=true \
  --set redis.enabled=true \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=coreauth.example.com
```

### On-Premise (Docker Compose)

Self-hosted configuration is in `deploy/onprem/docker compose/`.

---

## Database Migrations

Migrations are in `coreauth-core/migrations/` (001-013). They run automatically on startup via `docker-entrypoint.sh`.

### Manual Migration

```bash
# Apply all migrations
psql $DATABASE_URL < coreauth-core/migrations/001_core.sql
psql $DATABASE_URL < coreauth-core/migrations/002_auth.sql
# ... through 013
```

---

## Health Checks

- **Backend:** `GET /health`
- **PostgreSQL:** `pg_isready -U coreauth`
- **Redis:** `redis-cli ping`

---

## Monitoring & Logging

```bash
# Docker Compose logs
docker compose logs -f coreauth-core
docker compose logs -f coreauth-ui

# Kubernetes
kubectl logs -f deployment/coreauth-core
```

---

## Backup & Recovery

### Database Backup

```bash
# Backup
docker compose exec postgres pg_dump -U coreauth coreauth > backup_$(date +%Y%m%d).sql

# Restore
docker compose exec -T postgres psql -U coreauth coreauth < backup.sql
```

---

## Security Checklist

- [ ] Change all default passwords and JWT secret
- [ ] Use HTTPS/TLS everywhere
- [ ] Configure firewall rules (only expose ports 80/443)
- [ ] Enable rate limiting
- [ ] Set up monitoring and alerting
- [ ] Apply regular security updates
- [ ] Enable database encryption at rest
- [ ] Enable audit logging
- [ ] Enforce MFA for admin users
- [ ] Set strong password policies

---

## Scaling

### Horizontal

- Run multiple backend instances behind a load balancer
- Use shared Redis for session storage
- Database read replicas for read-heavy workloads

### Vertical

- Increase container CPU/memory limits
- Scale PostgreSQL instance size
- Increase Redis memory

---

## Rolling Updates

```bash
# Rebuild and restart
docker compose up -d --no-deps --build coreauth-core

# Verify health
curl http://localhost:8000/health
```
