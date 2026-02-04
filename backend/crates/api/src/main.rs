// CIAM API Server
// Main entry point for the IAM system REST API

mod config;
mod handlers;
mod middleware;
mod routes;
pub mod webhook_events;

use config::Config;
use dotenvy::dotenv;
use std::sync::Arc;
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, Any},
};

pub struct AppState {
    pub auth_service: ciam_auth::AuthService,
    pub oidc_service: ciam_auth::OidcService,
    pub oauth2_service: Arc<ciam_auth::OAuth2Service>,
    pub verification_service: Arc<ciam_auth::VerificationService>,
    pub password_reset_service: Arc<ciam_auth::PasswordResetService>,
    pub invitation_service: Arc<ciam_auth::InvitationService>,
    pub audit_service: Arc<ciam_auth::AuditService>,
    pub application_service: ciam_authz::ApplicationService,
    pub tuple_service: ciam_authz::TupleService,
    pub policy_engine: ciam_authz::PolicyEngine,
    pub cache: Arc<ciam_cache::Cache>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,ciam_api=debug,tower_http=debug".to_string()),
        )
        .init();

    tracing::info!("🚀 Starting CIAM API Server");
    tracing::info!("📦 Version: {}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::from_env();
    tracing::info!("🌍 Environment: {}", std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()));
    tracing::info!("🔌 Server: {}:{}", config.server_host, config.server_port);

    // Initialize database
    tracing::info!("🗄️  Connecting to database...");
    let database = ciam_database::Database::new(config.database.clone())
        .await
        .expect("Failed to connect to database");
    database.ping().await.expect("Database ping failed");
    tracing::info!("✅ Database connected");

    // Initialize cache
    tracing::info!("⚡ Connecting to Redis...");
    let cache = ciam_cache::Cache::new(config.cache.clone())
        .await
        .expect("Failed to connect to Redis");
    cache.ping().await.expect("Redis ping failed");
    tracing::info!("✅ Redis connected");

    // Initialize JWT service
    let jwt_service = ciam_auth::JwtService::from_env();
    tracing::info!("🔐 JWT service initialized");

    // Create auth service
    let auth_service = ciam_auth::AuthService::new(database.clone(), cache.clone(), jwt_service);
    tracing::info!("🔑 Auth service initialized");

    // Create OIDC service
    let oidc_service = ciam_auth::OidcService::new(database.clone(), cache.clone());
    tracing::info!("🔗 OIDC service initialized");

    // Create OAuth2 authorization server service
    let issuer = std::env::var("ISSUER_URL")
        .unwrap_or_else(|_| format!("http://{}:{}", config.server_host, config.server_port));
    let oauth2_service = Arc::new(
        ciam_auth::OAuth2Service::new(Arc::new(database.clone()), issuer)
            .await
            .expect("Failed to initialize OAuth2 service")
    );
    tracing::info!("🔐 OAuth2 authorization server initialized");

    // Create email service
    let email_service = ciam_auth::EmailService::from_env()
        .expect("Failed to initialize email service");
    tracing::info!("📧 Email service initialized");

    // Create verification service
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let verification_service = Arc::new(ciam_auth::VerificationService::new(
        database.clone(),
        email_service.clone(),
        base_url.clone(),
    ));
    tracing::info!("✉️  Verification service initialized");

    // Create password reset service
    let password_reset_service = Arc::new(ciam_auth::PasswordResetService::new(
        database.clone(),
        email_service.clone(),
        base_url.clone(),
    ));
    tracing::info!("🔑 Password reset service initialized");

    // Create invitation service
    let invitation_service = Arc::new(ciam_auth::InvitationService::new(
        database.clone(),
        email_service,
        base_url,
    ));
    tracing::info!("📬 Invitation service initialized");

    // Create audit service
    let audit_service = Arc::new(ciam_auth::AuditService::new(database.pool().clone()));
    tracing::info!("📋 Audit service initialized");

    // Create authz services
    let application_service = ciam_authz::ApplicationService::new(database.pool().clone());
    tracing::info!("🔐 Application service initialized");

    let tuple_service = ciam_authz::TupleService::new(database.pool().clone());
    tracing::info!("📊 Tuple service initialized");

    let policy_engine = ciam_authz::PolicyEngine::new(
        tuple_service.clone(),
        cache.clone(),
    );
    tracing::info!("🛡️  Policy engine initialized");

    // Create app state
    let state = Arc::new(AppState {
        auth_service,
        oidc_service,
        oauth2_service,
        verification_service,
        password_reset_service,
        invitation_service,
        audit_service,
        application_service,
        tuple_service,
        policy_engine,
        cache: Arc::new(cache),
    });

    // Create router
    let app = routes::create_router(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    tracing::info!("📡 Routes configured:");
    tracing::info!("   GET  /health");
    tracing::info!("   POST /api/auth/register");
    tracing::info!("   POST /api/auth/login");
    tracing::info!("   POST /api/auth/login-hierarchical");
    tracing::info!("   POST /api/auth/refresh");
    tracing::info!("   POST /api/auth/logout");
    tracing::info!("   GET  /api/auth/me");
    tracing::info!("   --- OAuth2/OIDC Authorization Server ---");
    tracing::info!("   GET  /.well-known/openid-configuration");
    tracing::info!("   GET  /.well-known/jwks.json");
    tracing::info!("   GET  /authorize");
    tracing::info!("   POST /oauth/token");
    tracing::info!("   GET  /userinfo");
    tracing::info!("   POST /oauth/revoke");
    tracing::info!("   POST /oauth/introspect");
    tracing::info!("   GET  /logout");
    tracing::info!("   --- Webhooks ---");
    tracing::info!("   GET/POST /api/organizations/:org_id/webhooks");
    tracing::info!("   GET/PUT/DELETE /api/organizations/:org_id/webhooks/:webhook_id");
    tracing::info!("   POST /api/organizations/:org_id/webhooks/:webhook_id/test");
    tracing::info!("   GET  /api/webhooks/event-types");

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("✅ Server ready at http://{}", addr);
    tracing::info!("🎯 Ready to accept requests!");

    axum::serve(listener, app)
        .await
        .expect("Server error");

    Ok(())
}
