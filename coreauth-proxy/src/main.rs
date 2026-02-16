mod auth;
mod config;
mod fga;
mod jwt;
mod middleware;
mod reverse_proxy;
mod route_matcher;
mod session;

use axum::body::Body;
use axum::http::Request;
use axum::Router;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;

use config::ProxyConfig;
use fga::FgaClient;
use jwt::JwtValidator;
use reverse_proxy::ReverseProxy;
use session::SessionManager;

/// CoreAuth Identity-Aware Reverse Proxy
#[derive(Parser)]
#[command(name = "coreauth-proxy", about = "Identity-aware reverse proxy with FGA")]
struct Cli {
    /// Path to the proxy configuration file
    #[arg(short, long, env = "CONFIG_PATH", default_value = "proxy.yaml")]
    config: PathBuf,
}

/// Shared application state available to all request handlers.
pub struct ProxyState {
    pub config: ProxyConfig,
    pub session_manager: SessionManager,
    pub jwt_validator: Arc<JwtValidator>,
    pub fga_client: FgaClient,
    pub reverse_proxy: ReverseProxy,
    pub coreauth_proxy: ReverseProxy,
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "coreauth_proxy=info,tower_http=info".into()),
        )
        .init();

    let cli = Cli::parse();

    // Load configuration
    let config = ProxyConfig::load(&cli.config)
        .unwrap_or_else(|e| {
            eprintln!("Failed to load config from {:?}: {}", cli.config, e);
            std::process::exit(1);
        });

    tracing::info!("CoreAuth Proxy starting");
    tracing::info!("  Upstream: {}", config.server.upstream);
    tracing::info!("  CoreAuth: {}", config.coreauth.url);
    tracing::info!("  Listen:   {}", config.server.listen);
    tracing::info!("  Routes:   {} rules", config.routes.len());

    let http_client = reqwest::Client::new();

    // Initialize JWT validator
    let jwt_validator = Arc::new(JwtValidator::new(&config.coreauth.url));

    // Fetch JWKS on startup
    if let Err(e) = jwt_validator.refresh_jwks().await {
        tracing::warn!("Initial JWKS fetch failed (will retry): {}", e);
    }

    // Start background JWKS refresh (every 5 minutes)
    jwt_validator.start_refresh_task(300);

    // Initialize components
    let session_manager = SessionManager::new(&config.session);
    let fga_client = FgaClient::new(
        &config.coreauth.url,
        &config.fga.store_name,
        http_client.clone(),
    );
    let reverse_proxy = ReverseProxy::new(&config.server.upstream);
    let coreauth_proxy = ReverseProxy::new(&config.coreauth.url);

    // Start background session cleanup (every 5 minutes)
    session_manager.start_cleanup_task(300);

    let listen_addr = config.server.listen.clone();

    let state = Arc::new(ProxyState {
        config,
        session_manager,
        jwt_validator,
        fga_client,
        reverse_proxy,
        coreauth_proxy,
        http_client,
    });

    // Build router — all requests go through our middleware handler
    let app = Router::new()
        .fallback(move |req: Request<Body>| {
            let state = Arc::clone(&state);
            async move {
                middleware::handle_request(state, req).await
            }
        });

    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {}: {}", listen_addr, e);
            std::process::exit(1);
        });

    tracing::info!("CoreAuth Proxy listening on {}", listen_addr);

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        });
}
