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
    pub connection_service: ciam_auth::ConnectionService,
    pub db: ciam_database::Database,
    pub jwt_service: ciam_auth::JwtService,
    pub email_service: Option<ciam_auth::EmailService>,
    pub sms_service: Option<ciam_auth::SmsService>,
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
    pub fga_store_service: ciam_authz::FgaStoreService,
    pub tenant_router: ciam_database::TenantDatabaseRouter,
    pub cache: Arc<ciam_cache::Cache>,
    pub self_service_service: Arc<ciam_auth::SelfServiceFlowService>,
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

    // Initialize JWT service with RS256 support for OAuth2-issued tokens
    let jwt_service = {
        let mut svc = ciam_auth::JwtService::from_env();
        // Load the RS256 public key from the signing_keys table
        let rs256_pub: Option<(String,)> = sqlx::query_as(
            "SELECT public_key_pem FROM signing_keys WHERE is_current = true LIMIT 1"
        )
        .fetch_optional(database.pool())
        .await
        .ok()
        .flatten();
        if let Some((pem,)) = rs256_pub {
            svc = svc.with_rs256_public_key(&pem);
            tracing::info!("🔐 JWT service initialized (HS256 + RS256)");
        } else {
            tracing::info!("🔐 JWT service initialized (HS256 only)");
        }
        svc
    };

    // Create auth service (clone jwt_service since we need it later for passwordless)
    let auth_service = ciam_auth::AuthService::new(database.clone(), cache.clone(), jwt_service.clone());
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

    // Create email service (optional - may not be configured in dev)
    let email_service = ciam_auth::EmailService::from_env().ok();
    if email_service.is_some() {
        tracing::info!("📧 Email service initialized");
    } else {
        tracing::warn!("📧 Email service not configured - passwordless emails will be logged only");
    }

    // Create SMS service (optional - may not be configured in dev)
    let sms_service = ciam_auth::SmsService::from_env().ok();
    if sms_service.is_some() {
        tracing::info!("📱 SMS service initialized");
    } else {
        tracing::warn!("📱 SMS service not configured - SMS MFA will not be available");
    }

    // Create verification service
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Clone email service for other services (they require non-Option)
    let email_service_for_verification = email_service.clone()
        .unwrap_or_else(|| ciam_auth::EmailService::console_only());
    let email_service_for_reset = email_service_for_verification.clone();
    let email_service_for_invitation = email_service_for_verification.clone();

    let verification_service = Arc::new(ciam_auth::VerificationService::new(
        database.clone(),
        email_service_for_verification,
        base_url.clone(),
    ));
    tracing::info!("✉️  Verification service initialized");

    // Create password reset service
    let password_reset_service = Arc::new(ciam_auth::PasswordResetService::new(
        database.clone(),
        email_service_for_reset,
        base_url.clone(),
    ));
    tracing::info!("🔑 Password reset service initialized");

    // Create invitation service
    let invitation_service = Arc::new(ciam_auth::InvitationService::new(
        database.clone(),
        email_service_for_invitation,
        base_url.clone(),
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

    // Create FGA store service
    let fga_store_service = ciam_authz::FgaStoreService::new(database.pool().clone());
    tracing::info!("📦 FGA store service initialized");

    // Create tenant database router for multi-tenant isolation
    let tenant_router_config = ciam_database::TenantRouterConfig::from_env();
    let tenant_router = ciam_database::TenantDatabaseRouter::new(
        database.pool().clone(),
        tenant_router_config,
    );
    tracing::info!("🏢 Tenant database router initialized");

    // Create self-service flow service
    let cache = Arc::new(cache);
    let self_service_service = Arc::new(ciam_auth::SelfServiceFlowService::new(
        Arc::new(database.clone()),
        cache.clone(),
        oauth2_service.clone(),
    ));
    tracing::info!("🔄 Self-service flow service initialized");

    // Create app state
    let connection_service = ciam_auth::ConnectionService::new(database.clone());
    tracing::info!("🔌 Connection service initialized");

    let state = Arc::new(AppState {
        connection_service,
        db: database.clone(),
        jwt_service,
        email_service,
        sms_service,
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
        fga_store_service,
        tenant_router,
        cache,
        self_service_service,
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
