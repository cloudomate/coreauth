#!/bin/bash

# CoreAuth Unified Startup Script
# Supports: development (default), production, test

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default to development
ENVIRONMENT=${1:-development}

echo "🚀 Starting CoreAuth in ${ENVIRONMENT} mode..."
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not found. Install from https://docs.docker.com/get-docker/${NC}"
    exit 1
fi

if ! docker compose version &> /dev/null; then
    echo -e "${RED}❌ Docker Compose not found${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Docker & Docker Compose ready"

# Check .env file
if [ ! -f .env ]; then
    echo -e "${YELLOW}⚠${NC}  No .env file found. Creating from .env.example..."
    cp .env.example .env
    echo -e "${GREEN}✓${NC} Created .env file"
    echo -e "${YELLOW}⚠${NC}  Please edit .env with your configuration"
    echo ""
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}Building images...${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

docker compose build

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}Starting services...${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

docker compose up -d

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}Waiting for services...${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Wait for PostgreSQL
echo -n "⏳ Waiting for PostgreSQL... "
until docker compose exec -T postgres pg_isready -U coreauth > /dev/null 2>&1; do
    sleep 1
done
echo -e "${GREEN}✓${NC}"

# Wait for Redis
echo -n "⏳ Waiting for Redis... "
until docker compose exec -T redis redis-cli ping > /dev/null 2>&1; do
    sleep 1
done
echo -e "${GREEN}✓${NC}"

# Wait for backend
echo -n "⏳ Waiting for backend... "
sleep 5
echo -e "${GREEN}✓${NC}"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✨ CoreAuth is running!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}Services:${NC}"
echo "  🌐 Frontend:    http://localhost:3000"
echo "  🚀 Backend API: http://localhost:8000"
echo "  🗄️  PostgreSQL:  localhost:5432"
echo "  📦 Redis:       localhost:6379"
echo ""
echo -e "${BLUE}Quick Commands:${NC}"
echo "  View logs:       docker compose logs -f"
echo "  Backend logs:    docker compose logs -f backend"
echo "  Frontend logs:   docker compose logs -f frontend"
echo "  Stop:            docker compose stop"
echo "  Restart:         docker compose restart"
echo "  Clean up:        docker compose down -v"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo "  1. Visit http://localhost:3000/signup"
echo "  2. Create your first organization"
echo "  3. Login and explore!"
echo ""
