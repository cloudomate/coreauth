# Multi-stage build for CoreAuth Backend (Rust)

# Stage 1: Build
FROM rust:1.93-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Copy SQLx offline query cache
COPY .sqlx ./.sqlx

# Build dependencies (cached layer) with SQLx offline mode
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    curl \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 coreauth

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/coreauth-api /app/coreauth-api

# Copy migrations
COPY migrations /app/migrations

# Copy entrypoint script
COPY docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

# Change ownership
RUN chown -R coreauth:coreauth /app

# Switch to app user
USER coreauth

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=40s \
    CMD curl -f http://localhost:8000/health || exit 1

# Run the application with entrypoint
ENTRYPOINT ["/app/docker-entrypoint.sh"]
