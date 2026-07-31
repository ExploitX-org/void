use std::collections::BTreeMap;

use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::crypto;
use crate::error::ApiError;
use crate::routes;
use crate::signing;

/// Maps a request path to its corresponding signing key_id.
fn key_id_for_path(path: &str) -> Option<&'static str> {
  if path == routes::constants::HEALTH || path == "/health/" {
    Some("health")
  } else if path == routes::constants::REGISTER {
    Some("register")
  } else if path == routes::constants::VERIFY {
    Some("authn/verify")
  } else if path == routes::constants::RESEND {
    Some("authn/resend")
  } else if path == routes::constants::CREATE_PAYMENT {
    Some("pmt/create")
  } else if path == routes::constants::PAYMENT_STATUS {
    Some("pmt/status")
  } else if path == routes::constants::SANDBOX
    || path == "/sandbox/"
    || path == "/sandbox/openapi.json"
  {
    Some("sandbox")
  } else {
    None
  }
}

/// Looks up the signing secret for a given key_id.
fn lookup_secret<'a>(
  key_id: &str,
  config: &'a crate::config::Config,
) -> Option<&'a str> {
  match key_id {
    "health" => Some(&config.auth_hmac_secret_for_health),
    "register" => Some(&config.auth_hmac_secret_for_register),
    "authn/verify" => Some(&config.auth_hmac_secret_for_authn_verify),
    "authn/resend" => Some(&config.auth_hmac_secret_for_authn_resend),
    "pmt/create" => Some(&config.auth_hmac_secret_for_pmt_create),
    "pmt/status" => Some(&config.auth_hmac_secret_for_pmt_status),
    "sandbox" => Some(&config.auth_hmac_secret_for_sandbox),
    _ => None,
  }
}

pub async fn require_auth(
  State(state): State<AppState>,
  req: Request<Body>,
  next: Next,
) -> Response {
  // 1. Extract Authorization header
  let auth_header = req
    .headers()
    .get("authorization")
    .and_then(|v| v.to_str().ok());

  let Some(auth_header) = auth_header else {
    tracing::warn!("sigv4 rejection: missing Authorization header");
    return ApiError::Unauthorized("missing Authorization header".into())
      .into_response();
  };

  // 2. Parse Authorization header
  let parsed = match signing::parse_authorization_header(auth_header) {
    Ok(p) => p,
    Err(e) => {
      tracing::warn!(error = %e, "sigv4 rejection: malformed Authorization header");
      return ApiError::Unauthorized(format!("malformed Authorization: {e}"))
        .into_response();
    }
  };

  // 3. Extract key_id and lookup secret
  let key_id = &parsed.key_id;
  let Some(secret) = lookup_secret(key_id, &state.config) else {
    tracing::warn!(key_id = %key_id, "sigv4 rejection: unknown signing key");
    return ApiError::Unauthorized("unknown signing key".into())
      .into_response();
  };

  // 4. Extract and parse X-Timestamp
  let timestamp_str = req
    .headers()
    .get("x-timestamp")
    .and_then(|v| v.to_str().ok());

  let Some(timestamp_str) = timestamp_str else {
    tracing::warn!("sigv4 rejection: missing X-Timestamp header");
    return ApiError::Unauthorized("missing X-Timestamp header".into())
      .into_response();
  };

  let timestamp: i64 = match timestamp_str.parse() {
    Ok(t) => t,
    Err(_) => {
      tracing::warn!(value = %timestamp_str, "sigv4 rejection: invalid X-Timestamp");
      return ApiError::Unauthorized("invalid X-Timestamp".into())
        .into_response();
    }
  };

  // 5. Verify timestamp within tolerance
  if let Err(e) =
    signing::verify_timestamp(timestamp, state.verification.timestamp_ttl_secs)
  {
    tracing::warn!(timestamp = %timestamp, error = %e, "sigv4 rejection: timestamp expired or clock skew");
    return ApiError::Unauthorized(format!("timestamp: {e}")).into_response();
  }

  // 6. Extract X-Nonce
  let nonce_str = req.headers().get("x-nonce").and_then(|v| v.to_str().ok());

  let Some(nonce_str) = nonce_str else {
    tracing::warn!("sigv4 rejection: missing X-Nonce header");
    return ApiError::Unauthorized("missing X-Nonce header".into())
      .into_response();
  };

  if nonce_str.is_empty() {
    tracing::warn!("sigv4 rejection: empty X-Nonce");
    return ApiError::Unauthorized("empty X-Nonce".into()).into_response();
  }

  let nonce = nonce_str.to_string();

  // 7. Check nonce cache (reject duplicate)
  {
    let cache = state.nonce_cache.lock().expect("nonce cache lock poisoned");
    if cache.contains(&nonce) {
      tracing::warn!(nonce = %nonce, "sigv4 rejection: nonce replay detected");
      return ApiError::Unauthorized("nonce replay detected".into())
        .into_response();
    }
  }

  // 8. Extract path, query, and headers for canonical request
  let path = req.uri().path().to_string();
  let query = req.uri().query().unwrap_or("").to_string();

  // Determine key_id for path validation
  let Some(expected_key_id) = key_id_for_path(&path) else {
    tracing::warn!(path = %path, "sigv4 rejection: unsigned route");
    return ApiError::Unauthorized("unsigned route".into()).into_response();
  };

  if key_id != expected_key_id {
    tracing::warn!(
      key_id = %key_id,
      expected = %expected_key_id,
      path = %path,
      "sigv4 rejection: key_id does not match route"
    );
    return ApiError::Unauthorized("key_id does not match route".into())
      .into_response();
  }

  // Build BTreeMap of request headers (lowercased names)
  let mut headers_map = BTreeMap::new();
  for (name, value) in req.headers() {
    if let Ok(v) = value.to_str() {
      headers_map.insert(name.as_str().to_lowercase(), v.to_string());
    }
  }

  // Compute body hash
  let (parts, body) = req.into_parts();
  let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
    Ok(b) => b,
    Err(_) => {
      tracing::warn!("sigv4 rejection: failed to read request body");
      return ApiError::Unauthorized("failed to read request body".into())
        .into_response();
    }
  };
  let body_hash = crypto::sha256_hex(&body_bytes);

  // Reconstruct the request with the consumed body
  let req = Request::from_parts(parts, Body::from(body_bytes));

  // 9. Build canonical request and verify signature
  let canonical_request = signing::build_canonical_request(
    req.method().as_str(),
    &path,
    &query,
    &headers_map,
    &parsed.signed_headers,
    &body_hash,
  );

  let canonical_request_hash = crypto::sha256_hex(canonical_request.as_bytes());

  if let Err(e) = signing::verify_signature(
    secret,
    key_id,
    timestamp,
    &nonce,
    &canonical_request_hash,
    &parsed.signature,
  ) {
    tracing::warn!(
      key_id = %key_id,
      method = %req.method().as_str(),
      path = %path,
      error = %e,
      "sigv4 signature verification failed"
    );
    return ApiError::Unauthorized(format!("signature: {e}")).into_response();
  }

  // 10. Insert nonce into cache
  {
    let mut cache =
      state.nonce_cache.lock().expect("nonce cache lock poisoned");
    cache.insert(&nonce);
  }

  next.run(req).await
}
