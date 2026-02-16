use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use ciam_cache::Cache;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
struct RateLimitError {
    error: String,
    message: String,
    retry_after: u64,
}

pub struct RateLimiter {
    cache: Cache,
}

impl RateLimiter {
    pub fn new(cache: Cache) -> Self {
        Self { cache }
    }

    /// Check if request is rate limited
    /// Returns (is_allowed, retry_after_seconds)
    pub async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u32,
        window_seconds: u64,
    ) -> Result<(bool, Option<u64>), ciam_cache::CacheError> {
        let current_count_key = format!("rate_limit:{}:count", key);
        let window_start_key = format!("rate_limit:{}:window_start", key);

        // Get current count and window start
        let current_count: Option<String> = self.cache.get(&current_count_key).await?;
        let window_start: Option<String> = self.cache.get(&window_start_key).await?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match (current_count, window_start) {
            (Some(count_str), Some(start_str)) => {
                let count: u32 = count_str.parse().unwrap_or(0);
                let start: u64 = start_str.parse().unwrap_or(now);

                // Check if window has expired
                if now - start >= window_seconds {
                    // Reset window
                    let one = String::from("1");
                    let now_str = now.to_string();
                    self.cache
                        .set(&current_count_key, &one, Some(window_seconds as usize))
                        .await?;
                    self.cache
                        .set(&window_start_key, &now_str, Some(window_seconds as usize))
                        .await?;
                    Ok((true, None))
                } else if count >= max_requests {
                    // Rate limit exceeded
                    let retry_after = window_seconds - (now - start);
                    Ok((false, Some(retry_after)))
                } else {
                    // Increment count
                    let new_count = count + 1;
                    let count_str = new_count.to_string();
                    self.cache
                        .set(&current_count_key, &count_str, Some(window_seconds as usize))
                        .await?;
                    Ok((true, None))
                }
            }
            _ => {
                // First request in window
                let one = String::from("1");
                let now_str = now.to_string();
                self.cache
                    .set(&current_count_key, &one, Some(window_seconds as usize))
                    .await?;
                self.cache
                    .set(&window_start_key, &now_str, Some(window_seconds as usize))
                    .await?;
                Ok((true, None))
            }
        }
    }
}

/// Extract IP address from request headers
fn extract_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .or_else(|| headers.get("x-real-ip").and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown")
        .trim()
        .to_string()
}

/// Rate limit middleware for login attempts
/// 5 requests per 60 seconds per IP
pub async fn rate_limit_login(
    State(cache): State<Arc<Cache>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let ip = extract_ip(request.headers());
    let rate_limiter = RateLimiter::new((*cache).clone());

    match rate_limiter.check_rate_limit(&format!("login:{}", ip), 5, 60).await {
        Ok((true, _)) => Ok(next.run(request).await),
        Ok((false, Some(retry_after))) => {
            tracing::warn!("Rate limit exceeded for login from IP: {}", ip);
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(RateLimitError {
                    error: "rate_limit_exceeded".to_string(),
                    message: format!("Too many login attempts. Please try again in {} seconds.", retry_after),
                    retry_after,
                }),
            )
                .into_response())
        }
        Ok((false, None)) => {
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(RateLimitError {
                    error: "rate_limit_exceeded".to_string(),
                    message: "Too many login attempts. Please try again later.".to_string(),
                    retry_after: 60,
                }),
            )
                .into_response())
        }
        Err(e) => {
            tracing::error!("Rate limit check error: {}", e);
            // On error, allow the request (fail open)
            Ok(next.run(request).await)
        }
    }
}

/// Rate limit middleware for registration
/// 3 requests per 300 seconds (5 minutes) per IP
pub async fn rate_limit_registration(
    State(cache): State<Arc<Cache>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let ip = extract_ip(request.headers());
    let rate_limiter = RateLimiter::new((*cache).clone());

    match rate_limiter.check_rate_limit(&format!("register:{}", ip), 3, 300).await {
        Ok((true, _)) => Ok(next.run(request).await),
        Ok((false, Some(retry_after))) => {
            tracing::warn!("Rate limit exceeded for registration from IP: {}", ip);
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(RateLimitError {
                    error: "rate_limit_exceeded".to_string(),
                    message: format!("Too many registration attempts. Please try again in {} seconds.", retry_after),
                    retry_after,
                }),
            )
                .into_response())
        }
        Ok((false, None)) => {
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(RateLimitError {
                    error: "rate_limit_exceeded".to_string(),
                    message: "Too many registration attempts. Please try again later.".to_string(),
                    retry_after: 300,
                }),
            )
                .into_response())
        }
        Err(e) => {
            tracing::error!("Rate limit check error: {}", e);
            Ok(next.run(request).await)
        }
    }
}

/// Rate limit middleware for password reset requests
/// 3 requests per 600 seconds (10 minutes) per IP
pub async fn rate_limit_password_reset(
    State(cache): State<Arc<Cache>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let ip = extract_ip(request.headers());
    let rate_limiter = RateLimiter::new((*cache).clone());

    match rate_limiter.check_rate_limit(&format!("password_reset:{}", ip), 3, 600).await {
        Ok((true, _)) => Ok(next.run(request).await),
        Ok((false, Some(retry_after))) => {
            tracing::warn!("Rate limit exceeded for password reset from IP: {}", ip);
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(RateLimitError {
                    error: "rate_limit_exceeded".to_string(),
                    message: format!("Too many password reset requests. Please try again in {} seconds.", retry_after),
                    retry_after,
                }),
            )
                .into_response())
        }
        Ok((false, None)) => {
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(RateLimitError {
                    error: "rate_limit_exceeded".to_string(),
                    message: "Too many password reset requests. Please try again later.".to_string(),
                    retry_after: 600,
                }),
            )
                .into_response())
        }
        Err(e) => {
            tracing::error!("Rate limit check error: {}", e);
            Ok(next.run(request).await)
        }
    }
}
