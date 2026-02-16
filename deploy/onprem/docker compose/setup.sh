#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# CoreAuth On-Prem Setup
# One-command installer for headless API deployment.
#
# Usage:
#   ./setup.sh                  # API only (headless)
#   ./setup.sh --with-dashboard # API + admin dashboard
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env"
TEMPLATE_FILE="${SCRIPT_DIR}/.env.onprem.template"
WITH_DASHBOARD=false

# ── Parse args ───────────────────────────────────────────────
for arg in "$@"; do
  case "$arg" in
    --with-dashboard) WITH_DASHBOARD=true ;;
    --help|-h)
      echo "Usage: ./setup.sh [--with-dashboard]"
      echo ""
      echo "Options:"
      echo "  --with-dashboard  Include admin dashboard UI"
      echo "  --help, -h        Show this help"
      exit 0
      ;;
    *)
      echo "Unknown option: $arg"
      echo "Run ./setup.sh --help for usage."
      exit 1
      ;;
  esac
done

# ── Helpers ──────────────────────────────────────────────────
info()  { echo "  [*] $*"; }
ok()    { echo "  [+] $*"; }
err()   { echo "  [!] $*" >&2; }
fatal() { err "$@"; exit 1; }

generate_secret() {
  local len="${1:-64}"
  openssl rand -base64 "$len" 2>/dev/null | tr -d '\n/+==' | head -c "$len"
}

generate_password() {
  openssl rand -base64 24 2>/dev/null | tr -d '\n/+' | head -c 16
}

# ── Prerequisites ────────────────────────────────────────────
echo ""
echo "============================================"
echo "  CoreAuth On-Prem Setup"
echo "============================================"
echo ""

info "Checking prerequisites..."

command -v docker >/dev/null 2>&1 || fatal "Docker is not installed. Please install Docker first."
docker compose version >/dev/null 2>&1 || fatal "'docker compose' not available. Please install Docker Compose v2."
command -v openssl >/dev/null 2>&1 || fatal "openssl is not installed."

ok "Prerequisites met."
echo ""

# ── Generate .env ────────────────────────────────────────────
if [ -f "$ENV_FILE" ]; then
  info "Existing .env found — preserving it."
  info "To regenerate, delete .env and re-run this script."
else
  info "Creating .env from template..."

  if [ ! -f "$TEMPLATE_FILE" ]; then
    fatal "Template not found: ${TEMPLATE_FILE}"
  fi

  JWT_SECRET=$(generate_secret 64)
  ADMIN_PASS=$(generate_password)

  cp "$TEMPLATE_FILE" "$ENV_FILE"

  # Replace generated placeholders
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s|JWT_SECRET=__GENERATE__|JWT_SECRET=${JWT_SECRET}|" "$ENV_FILE"
    sed -i '' "s|ADMIN_PASSWORD=__GENERATE__|ADMIN_PASSWORD=${ADMIN_PASS}|" "$ENV_FILE"
  else
    sed -i "s|JWT_SECRET=__GENERATE__|JWT_SECRET=${JWT_SECRET}|" "$ENV_FILE"
    sed -i "s|ADMIN_PASSWORD=__GENERATE__|ADMIN_PASSWORD=${ADMIN_PASS}|" "$ENV_FILE"
  fi

  ok "Generated .env with secrets."
  echo ""
  echo "  !! IMPORTANT: Edit .env before continuing !!"
  echo "  You MUST configure:"
  echo "    - POSTGRES_HOST / POSTGRES_PASSWORD"
  echo "    - REDIS_URL"
  echo "    - TENANT_NAME / TENANT_SLUG"
  echo "    - ADMIN_EMAIL"
  echo "    - APP_CALLBACK_URLS"
  echo ""
  read -rp "  Press Enter when .env is configured (or Ctrl+C to abort)..."
  echo ""
fi

# ── Load env ─────────────────────────────────────────────────
set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

# ── Validate required vars ───────────────────────────────────
info "Validating configuration..."

MISSING=""
[ -z "${POSTGRES_HOST:-}" ] && MISSING="${MISSING} POSTGRES_HOST"
[ -z "${POSTGRES_PASSWORD:-}" ] && MISSING="${MISSING} POSTGRES_PASSWORD"
[ -z "${REDIS_URL:-}" ] && MISSING="${MISSING} REDIS_URL"
[ -z "${JWT_SECRET:-}" ] && MISSING="${MISSING} JWT_SECRET"
[ -z "${TENANT_SLUG:-}" ] && MISSING="${MISSING} TENANT_SLUG"
[ -z "${ADMIN_EMAIL:-}" ] && MISSING="${MISSING} ADMIN_EMAIL"
[ -z "${ADMIN_PASSWORD:-}" ] && MISSING="${MISSING} ADMIN_PASSWORD"

if [ -n "$MISSING" ]; then
  fatal "Missing required variables in .env:${MISSING}"
fi

# Check for placeholder values
if [ "${POSTGRES_HOST}" = "your-db-host" ]; then
  fatal "POSTGRES_HOST is still set to placeholder 'your-db-host'. Please configure it in .env"
fi
if [ "${REDIS_URL}" = "redis://your-redis-host:6379" ]; then
  fatal "REDIS_URL is still set to placeholder. Please configure it in .env"
fi

ok "Configuration valid."

# ── Validate connectivity ────────────────────────────────────
info "Testing PostgreSQL connectivity (${POSTGRES_HOST}:${POSTGRES_PORT:-5432})..."
if docker run --rm --network=host postgres:16-alpine \
  pg_isready -h "${POSTGRES_HOST}" -p "${POSTGRES_PORT:-5432}" -U "${POSTGRES_USER:-coreauth}" \
  >/dev/null 2>&1; then
  ok "PostgreSQL is reachable."
else
  err "Cannot reach PostgreSQL at ${POSTGRES_HOST}:${POSTGRES_PORT:-5432}"
  err "Make sure your database is running and accessible."
  read -rp "  Continue anyway? (y/N) " CONTINUE
  [ "$CONTINUE" = "y" ] || [ "$CONTINUE" = "Y" ] || exit 1
fi

# Extract Redis host/port for connectivity check
REDIS_HOST_PORT=$(echo "${REDIS_URL}" | sed -n 's|redis://\([^/]*\).*|\1|p')
if [ -n "$REDIS_HOST_PORT" ]; then
  REDIS_HOST=$(echo "$REDIS_HOST_PORT" | cut -d: -f1)
  REDIS_PORT=$(echo "$REDIS_HOST_PORT" | cut -d: -f2)
  info "Testing Redis connectivity (${REDIS_HOST}:${REDIS_PORT:-6379})..."
  if docker run --rm --network=host redis:7-alpine \
    redis-cli -h "${REDIS_HOST}" -p "${REDIS_PORT:-6379}" ping \
    >/dev/null 2>&1; then
    ok "Redis is reachable."
  else
    err "Cannot reach Redis at ${REDIS_HOST}:${REDIS_PORT:-6379}"
    read -rp "  Continue anyway? (y/N) " CONTINUE
    [ "$CONTINUE" = "y" ] || [ "$CONTINUE" = "Y" ] || exit 1
  fi
fi

echo ""

# ── Build compose command ────────────────────────────────────
COMPOSE_CMD="docker compose --env-file ${ENV_FILE} -f ${SCRIPT_DIR}/docker-compose.onprem.yml"

if [ "$WITH_DASHBOARD" = true ]; then
  COMPOSE_CMD="${COMPOSE_CMD} -f ${SCRIPT_DIR}/docker-compose.dashboard.yml"
  info "Mode: API + Dashboard"
else
  info "Mode: Headless API only"
fi

# ── Build and start ──────────────────────────────────────────
info "Building containers..."
$COMPOSE_CMD build --quiet

info "Starting CoreAuth..."
$COMPOSE_CMD up -d

echo ""
info "Waiting for backend to become healthy..."

# Wait for backend health (max 120s)
TIMEOUT=120
ELAPSED=0
while [ $ELAPSED -lt $TIMEOUT ]; do
  STATUS=$(docker inspect --format='{{.State.Health.Status}}' coreauth-core 2>/dev/null || echo "unknown")
  if [ "$STATUS" = "healthy" ]; then
    break
  fi
  sleep 2
  ELAPSED=$((ELAPSED + 2))
  printf "\r  [*] Waiting... (%ds)" "$ELAPSED"
done
printf "\r"

if [ "$STATUS" != "healthy" ]; then
  err "Backend did not become healthy within ${TIMEOUT}s."
  err "Check logs: docker logs coreauth-core"
  exit 1
fi

ok "Backend is healthy."
echo ""

# ── Wait for bootstrap ───────────────────────────────────────
info "Waiting for bootstrap to complete..."

TIMEOUT=60
ELAPSED=0
while [ $ELAPSED -lt $TIMEOUT ]; do
  BS_STATUS=$(docker inspect --format='{{.State.Status}}' coreauth-bootstrap 2>/dev/null || echo "unknown")
  if [ "$BS_STATUS" = "exited" ]; then
    break
  fi
  sleep 2
  ELAPSED=$((ELAPSED + 2))
done

BS_EXIT=$(docker inspect --format='{{.State.ExitCode}}' coreauth-bootstrap 2>/dev/null || echo "1")

if [ "$BS_EXIT" != "0" ]; then
  err "Bootstrap failed (exit code: ${BS_EXIT})."
  err "Check logs: docker logs coreauth-bootstrap"
  exit 1
fi

echo ""

# ── Print bootstrap output ───────────────────────────────────
docker logs coreauth-bootstrap 2>&1 | tail -20

echo ""
if [ "$WITH_DASHBOARD" = true ]; then
  info "Dashboard: http://localhost:${FRONTEND_PORT:-8080}"
fi
info "API:       http://localhost:${BACKEND_PORT:-3000}"
info "Health:    http://localhost:${BACKEND_PORT:-3000}/health"
echo ""
ok "CoreAuth is ready."
echo ""
