use crate::config::RouteRule;
use std::collections::HashMap;

/// Matches a request path + method against a list of route rules.
/// Returns the first matching rule and extracted path parameters.
pub fn match_route<'a>(
    rules: &'a [RouteRule],
    path: &str,
    method: &str,
) -> Option<(&'a RouteRule, HashMap<String, String>)> {
    for rule in rules {
        // Check method filter (empty = match all)
        if !rule.match_rule.methods.is_empty() {
            let method_upper = method.to_uppercase();
            if !rule.match_rule.methods.iter().any(|m| m.to_uppercase() == method_upper) {
                continue;
            }
        }

        // Check path pattern
        if let Some(params) = match_path(&rule.match_rule.path, path) {
            return Some((rule, params));
        }
    }
    None
}

/// Match a path pattern against a request path, extracting named parameters.
/// Supports:
///   - Exact: `/health`
///   - Parameters: `/api/projects/:id` → extracts `id`
///   - Single wildcard: `/public/*`
///   - Double wildcard: `/dashboard/**` (matches any depth)
fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    let mut params = HashMap::new();
    let mut pi = 0; // pattern index
    let mut ri = 0; // request path index

    while pi < pattern_parts.len() {
        let pp = pattern_parts[pi];

        if pp == "**" {
            // Double wildcard matches everything remaining
            return Some(params);
        }

        if ri >= path_parts.len() {
            // Pattern has more parts than path
            return None;
        }

        if pp == "*" {
            // Single wildcard matches one segment
            pi += 1;
            ri += 1;
            continue;
        }

        if let Some(param_name) = pp.strip_prefix(':') {
            // Named parameter — capture value
            params.insert(param_name.to_string(), path_parts[ri].to_string());
            pi += 1;
            ri += 1;
            continue;
        }

        // Exact match
        if pp != path_parts[ri] {
            return None;
        }

        pi += 1;
        ri += 1;
    }

    // Both must be exhausted (unless pattern ended with **)
    if ri == path_parts.len() {
        Some(params)
    } else {
        None
    }
}

/// Extract the object ID from the request based on the FGA rule's `object_id` spec.
/// Format: "path:<param>", "query:<param>", "header:<name>"
pub fn extract_object_id(
    spec: &str,
    path_params: &HashMap<String, String>,
    query_string: Option<&str>,
    headers: &axum::http::HeaderMap,
) -> Option<String> {
    if let Some(param) = spec.strip_prefix("path:") {
        return path_params.get(param).cloned();
    }

    if let Some(param) = spec.strip_prefix("query:") {
        if let Some(qs) = query_string {
            for pair in qs.split('&') {
                let mut kv = pair.splitn(2, '=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    if key == param {
                        return Some(value.to_string());
                    }
                }
            }
        }
        return None;
    }

    if let Some(header_name) = spec.strip_prefix("header:") {
        return headers
            .get(header_name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(match_path("/health", "/health").is_some());
        assert!(match_path("/health", "/healths").is_none());
        assert!(match_path("/api/v1", "/api/v1").is_some());
    }

    #[test]
    fn test_param_match() {
        let params = match_path("/api/projects/:id", "/api/projects/123").unwrap();
        assert_eq!(params.get("id").unwrap(), "123");
    }

    #[test]
    fn test_double_wildcard() {
        assert!(match_path("/dashboard/**", "/dashboard").is_some());
        assert!(match_path("/dashboard/**", "/dashboard/settings").is_some());
        assert!(match_path("/dashboard/**", "/dashboard/a/b/c").is_some());
        assert!(match_path("/other/**", "/dashboard").is_none());
    }

    #[test]
    fn test_single_wildcard() {
        assert!(match_path("/api/*/list", "/api/users/list").is_some());
        assert!(match_path("/api/*/list", "/api/users/detail").is_none());
    }
}
