use http::{HeaderName, Uri};
use once_cell::sync::Lazy;
use pingora::http::RequestHeader;
use pingora_proxy::Session;
use regex::Regex;
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};
use url::form_urlencoded;

use crate::config::UpstreamHashOn;

#[derive(Debug, PartialEq)]
pub enum PathMatch {
    Sse(String),            // match /api/{tenant_id}/sse
    Messages(String),       // match /api/{tenant_id}/messages
    StreamableHttp(String), // match /api/{tenant_id}/mcp
    NoMatch,                // match failed
}

// 使用 Lazy 初始化正则表达式
static API_SSE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^/api/(?P<tenant_id>[^/]+)/sse/?$").unwrap());
static API_MESSAGE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^/api/(?P<tenant_id>[^/]+)/messages/?$").unwrap());
static API_MCP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^/api/(?P<tenant_id>[^/]+)/mcp/?$").unwrap());

pub fn match_api_path(path: &str) -> PathMatch {
    log::debug!("match_api_path: {}", path);
    if let Some(caps) = API_SSE_RE.captures(path) {
        PathMatch::Sse(caps["tenant_id"].to_string())
    } else if let Some(caps) = API_MESSAGE_RE.captures(path) {
        PathMatch::Messages(caps["tenant_id"].to_string())
    } else if let Some(caps) = API_MCP_RE.captures(path) {
        PathMatch::StreamableHttp(caps["tenant_id"].to_string())
    } else {
        PathMatch::NoMatch
    }
}
/// Helper function to detect initialize requests
pub fn is_initialize_request(body: &Value) -> bool {
    match body {
        Value::Array(arr) => arr.iter().any(|msg| {
            if let Value::Object(obj) = msg {
                obj.get("method").and_then(|m| m.as_str()) == Some("initialize")
            } else {
                false
            }
        }),
        Value::Object(obj) => obj.get("method").and_then(|m| m.as_str()) == Some("initialize"),
        _ => false,
    }
}
pub fn extract_tenant_id(path: &str) -> Option<String> {
    let re = Regex::new(r"^/api/(?P<tenant_id>[^/?]+)/sse/?(\?.*)?$").unwrap();
    re.captures(path).map(|caps| caps["tenant_id"].to_string())
}

pub fn query_to_map(uri: &Uri) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Some(query) = uri.query() {
        for pair in query.split('&') {
            let mut kv = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                map.insert(key.to_string(), value.to_string());
            }
        }
    }

    map
}

pub fn replace_dynamic_params(path: &str, params: &Value) -> String {
    let re = Regex::new(r"\{(\w+)\}").unwrap();
    re.replace_all(path, |caps: &regex::Captures<'_>| {
        let key = &caps[1];
        let binding = Value::String("".to_string());
        let value = params.get(key).unwrap_or(&binding);
        // params.get(key).unwrap_or(&"") //
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "".to_string(),
            _ => value.to_string(),
        }
    })
    .to_string()
}

pub fn flatten_json(prefix: &str, value: &Value, result: &mut HashMap<String, String>) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                let new_prefix = if prefix.is_empty() {
                    key.to_owned()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_json(&new_prefix, val, result);
            }
        }
        Value::Array(arr) => {
            for (index, val) in arr.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, index);
                flatten_json(&new_prefix, val, result);
            }
        }
        Value::String(s) => {
            result.insert(prefix.to_string(), s.to_string());
        }
        Value::Number(n) => {
            result.insert(prefix.to_string(), n.to_string());
        }
        Value::Bool(b) => {
            result.insert(prefix.to_string(), b.to_string());
        }
        Value::Null => {} //
    }
}

pub fn merge_path_query(path: &str, query: &str) -> String {
    if query.is_empty() {
        return path.to_string();
    }
    if path.contains('?') {
        format!("{}&{}", path, query)
    } else {
        format!("{}?{}", path, query)
    }
}
pub fn json_to_uri_query(value: &Value) -> String {
    let mut flattened_params = HashMap::new();
    flatten_json("", value, &mut flattened_params);
    let mut new_query = "?".to_string();
    new_query.push_str(
        &form_urlencoded::Serializer::new(String::new())
            .extend_pairs(flattened_params.iter())
            .finish(),
    );
    new_query
}
/// Build request selector key.
pub fn request_selector_key(session: &mut Session, hash_on: &UpstreamHashOn, key: &str) -> String {
    match hash_on {
        UpstreamHashOn::VARS => handle_vars(session, key),
        UpstreamHashOn::HEAD => get_req_header_value(session.req_header(), key)
            .unwrap_or_default()
            .to_string(),
        UpstreamHashOn::COOKIE => get_cookie_value(session.req_header(), key)
            .unwrap_or_default()
            .to_string(),
    }
}

/// Handles variable-based request selection.
fn handle_vars(session: &mut Session, key: &str) -> String {
    if key.starts_with("arg_") {
        if let Some(name) = key.strip_prefix("arg_") {
            return get_query_value(session.req_header(), name)
                .unwrap_or_default()
                .to_string();
        }
    }

    match key {
        "uri" => session.req_header().uri.path().to_string(),
        "request_uri" => session
            .req_header()
            .uri
            .path_and_query()
            .map_or_else(|| "".to_string(), |pq| pq.to_string()),
        "query_string" => session
            .req_header()
            .uri
            .query()
            .unwrap_or_default()
            .to_string(),
        "remote_addr" => session
            .client_addr()
            .map_or_else(|| "".to_string(), |addr| addr.to_string()),
        "remote_port" => session
            .client_addr()
            .and_then(|s| s.as_inet())
            .map_or_else(|| "".to_string(), |i| i.port().to_string()),
        "server_addr" => session
            .server_addr()
            .map_or_else(|| "".to_string(), |addr| addr.to_string()),
        _ => "".to_string(),
    }
}

pub fn get_query_value<'a>(req_header: &'a RequestHeader, name: &str) -> Option<&'a str> {
    if let Some(query) = req_header.uri.query() {
        for item in query.split('&') {
            if let Some((k, v)) = item.split_once('=') {
                if k == name {
                    return Some(v.trim());
                }
            }
        }
    }
    None
}

/// Remove query parameter from request header URI
///
/// # Arguments
/// * `req_header` - The HTTP request header to modify
/// * `name` - Name of the query parameter to remove
///
/// # Returns
/// Result indicating success or failure of the URI modification
pub fn remove_query_from_header(
    req_header: &mut RequestHeader,
    name: &str,
) -> Result<(), http::uri::InvalidUri> {
    if let Some(query) = req_header.uri.query() {
        let mut query_list = vec![];
        for item in query.split('&') {
            if let Some((k, v)) = item.split_once('=') {
                if k != name {
                    query_list.push(format!("{k}={v}"));
                }
            } else if item != name {
                query_list.push(item.to_string());
            }
        }
        let query = query_list.join("&");
        let mut new_path = req_header.uri.path().to_string();
        if !query.is_empty() {
            new_path = format!("{new_path}?{query}");
        }
        return new_path
            .parse::<http::Uri>()
            .map(|uri| req_header.set_uri(uri));
    }

    Ok(())
}

pub fn get_req_header_value<'a>(req_header: &'a RequestHeader, key: &str) -> Option<&'a str> {
    if let Some(value) = req_header.headers.get(key) {
        if let Ok(value) = value.to_str() {
            return Some(value);
        }
    }
    None
}

pub fn get_cookie_value<'a>(req_header: &'a RequestHeader, cookie_name: &str) -> Option<&'a str> {
    if let Some(cookie_value) = get_req_header_value(req_header, "Cookie") {
        for item in cookie_value.split(';') {
            if let Some((k, v)) = item.split_once('=') {
                if k == cookie_name {
                    return Some(v.trim());
                }
            }
        }
    }

    log::warn!("Cookie '{}' not found or malformed.", cookie_name);
    None
}

/// Retrieves the request host from the request header.
pub fn get_request_host(header: &RequestHeader) -> Option<&str> {
    if let Some(host) = header.uri.host() {
        return Some(host);
    }
    if let Some(host) = header.headers.get(http::header::HOST) {
        if let Ok(value) = host.to_str().map(|host| host.split(':').next()) {
            return value;
        }
    }
    None
}

static HTTP_HEADER_X_FORWARDED_FOR: Lazy<http::HeaderName> =
    Lazy::new(|| HeaderName::from_str("X-Forwarded-For").unwrap());

static HTTP_HEADER_X_REAL_IP: Lazy<http::HeaderName> =
    Lazy::new(|| HeaderName::from_str("X-Real-Ip").unwrap());

/// Get remote address from session.
fn get_remote_addr(session: &Session) -> Option<(String, u16)> {
    session
        .client_addr()
        .and_then(|addr| addr.as_inet())
        .map(|addr| (addr.ip().to_string(), addr.port()))
}

/// Gets client IP from `X-Forwarded-For`, `X-Real-IP`, or remote address.
pub fn get_client_ip(session: &Session) -> String {
    if let Some(value) = session.get_header(HTTP_HEADER_X_FORWARDED_FOR.clone()) {
        if let Ok(forwarded) = value.to_str() {
            if let Some(ip) = forwarded.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    if let Some(value) = session.get_header(HTTP_HEADER_X_REAL_IP.clone()) {
        if let Ok(real_ip) = value.to_str() {
            return real_ip.trim().to_string();
        }
    }

    if let Some((addr, _)) = get_remote_addr(session) {
        return addr;
    }

    "".to_string()
}
#[test]
fn test_extract_tenant_id() {
    let paths = vec![
        "/api/12345/sse",
        "/api/abc-xyz/sse/",
        "/api/user123/sse?param=value",
    ];

    for path in paths {
        let res = match extract_tenant_id(path) {
            Some(tenant_id) => true,
            None => false,
        };
        assert!(res, "Failed for path: {}", path);
    }
    let paths = vec!["/api/invalid_path", "/api/", "/sse"];
    for path in paths {
        let res = match extract_tenant_id(path) {
            Some(tenant_id) => true,
            None => false,
        };
        assert!(!res, "Failed for path: {}", path);
    }
}
#[test]
fn flatten_json_object_with_nested_structure_flattens_correctly() {
    let json_value = serde_json::json!({
        "name": "John",
        "age": 30,
        "address": {
            "city": "New York",
            "zip": "10001"
        },
        "hobbies": ["reading", "traveling"]
    });

    let mut result = HashMap::new();
    flatten_json("", &json_value, &mut result);

    let expected = HashMap::from([
        ("name".to_string(), "John".to_string()),
        ("age".to_string(), "30".to_string()),
        ("address.city".to_string(), "New York".to_string()),
        ("address.zip".to_string(), "10001".to_string()),
        ("hobbies[0]".to_string(), "reading".to_string()),
        ("hobbies[1]".to_string(), "traveling".to_string()),
    ]);

    assert_eq!(result, expected);
}

#[test]
fn flatten_json_empty_object_returns_empty_map() {
    let json_value = serde_json::json!({});
    let mut result = HashMap::new();
    flatten_json("", &json_value, &mut result);

    assert!(result.is_empty());
}

#[test]
fn flatten_json_empty_array_returns_empty_map() {
    let json_value = serde_json::json!([]);
    let mut result = HashMap::new();
    flatten_json("", &json_value, &mut result);

    assert!(result.is_empty());
}

#[test]
fn flatten_json_null_value_ignores_null() {
    let json_value = serde_json::json!(null);
    let mut result = HashMap::new();
    flatten_json("", &json_value, &mut result);

    assert!(result.is_empty());
}
