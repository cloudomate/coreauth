use axum::body::Body;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Request, Response, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::str::FromStr;

/// Headers that should NOT be forwarded between hops.
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

/// CoreAuth identity headers that the proxy injects — strip any incoming ones
/// to prevent spoofing from untrusted clients.
const IDENTITY_HEADERS: &[&str] = &[
    "x-coreauth-user-id",
    "x-coreauth-user-email",
    "x-coreauth-tenant-id",
    "x-coreauth-tenant-slug",
    "x-coreauth-role",
    "x-coreauth-is-platform-admin",
    "x-coreauth-token",
];

pub struct ReverseProxy {
    client: Client<hyper_util::client::legacy::connect::HttpConnector, Body>,
    upstream: String,
}

impl ReverseProxy {
    pub fn new(upstream: &str) -> Self {
        let client = Client::builder(TokioExecutor::new())
            .build_http();
        Self {
            client,
            upstream: upstream.trim_end_matches('/').to_string(),
        }
    }

    pub async fn forward(
        &self,
        req: Request<Body>,
        identity_headers: Option<HeaderMap>,
    ) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
        let method = req.method().clone();
        let path_and_query = req.uri().path_and_query()
            .map(|pq| pq.to_string())
            .unwrap_or_else(|| "/".to_string());

        let upstream_uri = format!("{}{}", self.upstream, path_and_query);

        // Collect the body so we can set Content-Length correctly
        let (parts, body) = req.into_parts();
        let body_bytes = axum::body::to_bytes(body, 1024 * 1024).await
            .unwrap_or_default();

        tracing::debug!(
            "Forwarding {} {} → {} (body: {} bytes)",
            method, path_and_query, upstream_uri, body_bytes.len()
        );

        // Rebuild the request with the collected body
        let mut builder = Request::builder()
            .method(parts.method)
            .uri(Uri::from_str(&upstream_uri)?);

        // Copy headers
        if let Some(headers) = builder.headers_mut() {
            for (name, value) in parts.headers.iter() {
                let name_str = name.as_str().to_lowercase();
                // Skip hop-by-hop headers
                if HOP_BY_HOP.contains(&name_str.as_str()) {
                    continue;
                }
                // Skip identity headers (prevent spoofing)
                if IDENTITY_HEADERS.contains(&name_str.as_str()) {
                    continue;
                }
                // Skip Host header (hyper sets the correct one for upstream)
                if name_str == "host" {
                    continue;
                }
                headers.insert(name.clone(), value.clone());
            }

            // Set correct Content-Length for the collected body
            if !body_bytes.is_empty() {
                headers.insert(
                    HeaderName::from_static("content-length"),
                    HeaderValue::from_str(&body_bytes.len().to_string()).unwrap(),
                );
            }

            // Inject identity headers from auth middleware
            if let Some(id_headers) = identity_headers {
                for (name, value) in id_headers.iter() {
                    headers.insert(name.clone(), value.clone());
                }
            }
        }

        let new_req = builder.body(Body::from(body_bytes))?;
        let response = self.client.request(new_req).await?;
        // Convert hyper::body::Incoming to axum::body::Body
        let (parts, incoming) = response.into_parts();
        Ok(Response::from_parts(parts, Body::new(incoming)))
    }
}

/// Build identity headers from session claims.
pub fn build_identity_headers(claims: &crate::session::SessionData) -> HeaderMap {
    let mut headers = HeaderMap::new();

    if let Ok(v) = HeaderValue::from_str(&claims.user_id) {
        headers.insert(HeaderName::from_static("x-coreauth-user-id"), v);
    }
    if let Ok(v) = HeaderValue::from_str(&claims.email) {
        headers.insert(HeaderName::from_static("x-coreauth-user-email"), v);
    }
    if let Some(ref tid) = claims.tenant_id {
        if let Ok(v) = HeaderValue::from_str(tid) {
            headers.insert(HeaderName::from_static("x-coreauth-tenant-id"), v);
        }
    }
    if let Some(ref slug) = claims.tenant_slug {
        if let Ok(v) = HeaderValue::from_str(slug) {
            headers.insert(HeaderName::from_static("x-coreauth-tenant-slug"), v);
        }
    }
    if let Some(ref role) = claims.role {
        if let Ok(v) = HeaderValue::from_str(role) {
            headers.insert(HeaderName::from_static("x-coreauth-role"), v);
        }
    }
    if let Ok(v) = HeaderValue::from_str(&claims.access_token) {
        headers.insert(HeaderName::from_static("x-coreauth-token"), v);
    }

    headers
}
