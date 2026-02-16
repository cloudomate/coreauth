# Deployment Guide

## Production Deployment

### Using Docker Compose (Recommended)

#### 1. Prepare Environment

```bash
# Create production environment file
cp .env.example .env

# Edit .env with production values
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
BACKEND_URL=https://api.your-domain.com

# Email (Production SMTP)
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
SMTP_FROM=noreply@your-domain.com

# SMS (Optional - Twilio)
SMS_PROVIDER=twilio
TWILIO_ACCOUNT_SID=your-account-sid
TWILIO_AUTH_TOKEN=your-auth-token
TWILIO_PHONE_NUMBER=+1234567890
```

#### 2. Deploy with Docker Compose

```bash
# Build production images
docker compose -f docker-compose.prod.yml build

# Start services
docker compose -f docker-compose.prod.yml up -d

# Check status
docker compose -f docker-compose.prod.yml ps

# View logs
docker compose -f docker-compose.prod.yml logs -f
```

#### 3. Run Migrations

```bash
docker compose exec backend ./docker-entrypoint.sh migrate
```

#### 4. Setup Reverse Proxy (Nginx)

```nginx
# /etc/nginx/sites-available/coreauth

upstream backend {
    server localhost:8000;
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

Enable and restart Nginx:

```bash
ln -s /etc/nginx/sites-available/coreauth /etc/nginx/sites-enabled/
nginx -t
systemctl restart nginx
```

### Cloud Deployment Options

#### AWS (ECS + RDS)

1. **Setup RDS PostgreSQL**
   - Engine: PostgreSQL 14+
   - Instance: db.t3.medium (minimum)
   - Storage: 50GB+ gp3
   - Backup retention: 7 days

2. **Setup ElastiCache Redis**
   - Engine: Redis 6+
   - Node type: cache.t3.micro

3. **Deploy to ECS**
   ```bash
   # Build and push images
   docker build -t your-registry/coreauth-core:latest coreauth-core/
   docker build -t your-registry/coreauth-ui:latest coreauth-portal/
   docker push your-registry/coreauth-core:latest
   docker push your-registry/coreauth-ui:latest

   # Create ECS task definition and service
   aws ecs create-task-definition --cli-input-json file://ecs-task-def.json
   aws ecs create-service --cluster coreauth --service-name coreauth-core --task-definition coreauth-core
   ```

#### Google Cloud (Cloud Run + Cloud SQL)

1. **Setup Cloud SQL**
   ```bash
   gcloud sql instances create coreauth-db \
     --database-version=POSTGRES_14 \
     --tier=db-f1-micro \
     --region=us-central1
   ```

2. **Deploy to Cloud Run**
   ```bash
   # Build and deploy backend
   gcloud builds submit --tag gcr.io/PROJECT_ID/coreauth-core coreauth-core/
   gcloud run deploy coreauth-core \
     --image gcr.io/PROJECT_ID/coreauth-core \
     --platform managed \
     --region us-central1 \
     --add-cloudsql-instances PROJECT_ID:us-central1:coreauth-db

   # Build and deploy frontend
   gcloud builds submit --tag gcr.io/PROJECT_ID/coreauth-ui coreauth-portal/
   gcloud run deploy coreauth-ui \
     --image gcr.io/PROJECT_ID/coreauth-ui \
     --platform managed \
     --region us-central1
   ```

#### DigitalOcean (App Platform)

1. **Create App via UI or CLI**
   ```bash
   doctl apps create --spec .do/app.yaml
   ```

2. **Configure Managed Database**
   - PostgreSQL 14
   - Redis 6
   - Link to app

## Kubernetes Deployment

### Using Helm

```bash
# Add CoreAuth Helm repo (if published)
helm repo add coreauth https://charts.coreauth.dev
helm repo update

# Install
helm install coreauth coreauth/coreauth \
  --set postgresql.enabled=true \
  --set redis.enabled=true \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=coreauth.example.com
```

### Manual Kubernetes Deployment

See `k8s/` directory for:
- `deployment.yaml` - Backend and Frontend deployments
- `service.yaml` - Services
- `ingress.yaml` - Ingress configuration
- `configmap.yaml` - Configuration
- `secrets.yaml` - Secrets (base64 encoded)

```bash
kubectl apply -f k8s/
```

## Database Migrations

### Running Migrations

```bash
# Using Docker
docker compose exec backend sqlx migrate run

# Directly with sqlx-cli
cargo install sqlx-cli
sqlx migrate run

# Manual
psql $DATABASE_URL < coreauth-core/migrations/001_init.sql
```

### Creating New Migrations

```bash
# Create new migration
sqlx migrate add <migration_name>

# Edit the generated SQL file
vim coreauth-core/migrations/<timestamp>_<migration_name>.sql

# Test migration
sqlx migrate run
```

## Monitoring & Logging

### Application Logs

```bash
# Docker Compose
docker compose logs -f backend
docker compose logs -f frontend

# Kubernetes
kubectl logs -f deployment/coreauth-core
kubectl logs -f deployment/coreauth-ui
```

### Health Checks

- Backend: `http://localhost:8000/health`
- Frontend: `http://localhost:3000/`
- Database: Check PostgreSQL connection
- Redis: Check Redis connection

### Metrics (Prometheus)

Backend exposes Prometheus metrics at `/metrics`:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'coreauth-core'
    static_configs:
      - targets: ['localhost:8000']
    metrics_path: '/metrics'
```

## Backup & Recovery

### Database Backup

```bash
# Automated daily backups
docker compose exec postgres pg_dump -U coreauth coreauth > backup_$(date +%Y%m%d).sql

# Restore
docker compose exec -T postgres psql -U coreauth coreauth < backup_20260202.sql
```

### Disaster Recovery

1. **Database restore from backup**
2. **Restore Redis cache** (optional, rebuilds automatically)
3. **Redeploy application containers**
4. **Verify all services healthy**

## Security Checklist

- [ ] Change all default passwords
- [ ] Use strong JWT secret (64+ characters)
- [ ] Enable HTTPS/TLS everywhere
- [ ] Configure firewall rules
- [ ] Enable rate limiting
- [ ] Setup monitoring and alerting
- [ ] Regular security updates
- [ ] Database encryption at rest
- [ ] Backup encryption
- [ ] Audit logging enabled
- [ ] MFA enforced for admin users

## Performance Tuning

### Database

```sql
-- Increase connection pool
ALTER SYSTEM SET max_connections = '200';

-- Optimize for SSDs
ALTER SYSTEM SET random_page_cost = '1.1';

-- Increase shared buffers
ALTER SYSTEM SET shared_buffers = '256MB';
```

### Redis

```conf
# redis.conf
maxmemory 256mb
maxmemory-policy allkeys-lru
```

### Backend

```env
# Increase worker threads
TOKIO_WORKER_THREADS=8

# Database connection pool
DATABASE_POOL_MAX_CONNECTIONS=20
```

## Troubleshooting

### Common Issues

**1. Database connection failed**
```bash
# Check database is running
docker compose ps postgres

# Verify connection string
docker compose exec backend env | grep DATABASE_URL

# Test connection
docker compose exec postgres psql -U coreauth -d coreauth
```

**2. Migration failed**
```bash
# Check migration status
sqlx migrate info

# Revert last migration
sqlx migrate revert

# Force migration
sqlx migrate run --force
```

**3. Redis connection failed**
```bash
# Check Redis is running
docker compose ps redis

# Test connection
docker compose exec redis redis-cli ping
```

## Scaling

### Horizontal Scaling

- Run multiple backend instances behind load balancer
- Use shared Redis for session storage
- Database read replicas for read-heavy workloads

### Vertical Scaling

- Increase container CPU/memory limits
- Scale PostgreSQL instance size
- Increase Redis memory

## Updates & Maintenance

### Rolling Updates

```bash
# Pull latest images
docker compose pull

# Recreate containers (zero downtime)
docker compose up -d --no-deps --build backend

# Verify health
curl http://localhost:8000/health
```

### Database Maintenance

```bash
# Vacuum database
docker compose exec postgres vacuumdb -U coreauth -d coreauth --analyze

# Reindex
docker compose exec postgres reindexdb -U coreauth -d coreauth
```
