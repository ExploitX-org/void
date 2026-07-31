use std::collections::{BTreeMap, HashMap};
use std::env;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::crypto;

pub const ALGORITHM: &str = "VOID4-HMAC-SHA256";

#[derive(Clone)]
pub struct VerificationConfig {
  pub timestamp_ttl_secs: u64,
  pub nonce_cache_max: usize,
}

impl VerificationConfig {
  pub fn from_env() -> Self {
    Self {
      timestamp_ttl_secs: env::var("SIGNING_TIMESTAMP_TTL_SECS")
        .unwrap_or_else(|_| "60".into())
        .trim()
        .parse()
        .expect("SIGNING_TIMESTAMP_TTL_SECS must be a valid integer"),
      nonce_cache_max: env::var("SIGNING_NONCE_CACHE_MAX")
        .unwrap_or_else(|_| "500000".into())
        .trim()
        .parse()
        .expect("SIGNING_NONCE_CACHE_MAX must be a valid integer"),
    }
  }
}

pub struct NonceCache {
  cache: HashMap<String, Instant>,
  max_entries: usize,
  ttl: Duration,
}

impl NonceCache {
  pub fn new(max_entries: usize, ttl: Duration) -> Self {
    Self {
      cache: HashMap::with_capacity(max_entries.min(1024)),
      max_entries,
      ttl,
    }
  }

  /// Returns true if the nonce was inserted (new). Returns false if it already
  /// existed or the cache is full.
  pub fn insert(&mut self, nonce: &str) -> bool {
    if self.cache.len() >= self.max_entries {
      return false;
    }
    let prev = self.cache.insert(nonce.to_string(), Instant::now());
    prev.is_none()
  }

  pub fn contains(&self, nonce: &str) -> bool {
    self.cache.contains_key(nonce)
  }

  pub fn cleanup(&mut self) {
    let cutoff = Instant::now() - self.ttl;
    self.cache.retain(|_, inserted| *inserted > cutoff);
  }
}

/// Parsed Authorization header components.
pub struct AuthorizationHeader {
  pub key_id: String,
  pub signed_headers: Vec<String>,
  pub signature: String,
}

/// Parses an `Authorization` header value.
///
/// Format: `VOID4-HMAC-SHA256 Credential={endpoint}, SignedHeaders={list}, Signature={sig}`
pub fn parse_authorization_header(
  header: &str,
) -> Result<AuthorizationHeader, &'static str> {
  let header = header.trim();
  if !header.starts_with("VOID4-HMAC-SHA256 ") {
    return Err("invalid algorithm prefix");
  }
  let rest = &header[ALGORITHM.len() + 1..];

  let mut key_id = None;
  let mut signed_headers = None;
  let mut signature = None;

  for part in rest.split(',').map(str::trim) {
    if let Some(val) = part.strip_prefix("Credential=") {
      key_id = Some(val.trim().to_string());
    } else if let Some(val) = part.strip_prefix("SignedHeaders=") {
      signed_headers = Some(
        val
          .split(';')
          .map(str::trim)
          .filter(|s| !s.is_empty())
          .map(str::to_string)
          .collect(),
      );
    } else if let Some(val) = part.strip_prefix("Signature=") {
      signature = Some(val.trim().to_string());
    }
  }

  let key_id = key_id.ok_or("missing Credential")?;
  let signed_headers = signed_headers.ok_or("missing SignedHeaders")?;
  let signature = signature.ok_or("missing Signature")?;

  if key_id.is_empty() {
    return Err("empty Credential");
  }
  if signature.is_empty() {
    return Err("empty Signature");
  }

  Ok(AuthorizationHeader {
    key_id,
    signed_headers,
    signature,
  })
}

/// Returns the current Unix epoch in seconds.
pub fn current_timestamp_secs() -> Result<i64, &'static str> {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs() as i64)
    .map_err(|_| "system clock before UNIX epoch")
}

/// Verifies that `timestamp` is within ±`tolerance` seconds of now.
pub fn verify_timestamp(
  timestamp: i64,
  tolerance: u64,
) -> Result<(), &'static str> {
  let now = current_timestamp_secs()?;
  let diff = now.abs_diff(timestamp);
  if diff > tolerance {
    return Err("timestamp expired or clock skew too large");
  }
  Ok(())
}

/// Builds the canonical request from the parsed Authorization components and
/// the actual request headers.
///
/// Canonical format:
/// ```text
/// {METHOD}\n
/// {PATH}\n
/// {QUERY}\n
/// {SORTED_LOWERCASED_HEADERS}\n
/// {SIGNED_HEADERS}\n
/// {SHA256_HEX(BODY)}
/// ```
///
/// Only headers listed in `signed_headers` are included in the canonical
/// headers section.
pub fn build_canonical_request(
  method: &str,
  path: &str,
  query: &str,
  headers: &BTreeMap<String, String>,
  signed_headers: &[String],
  body_hash: &str,
) -> String {
  let sorted_headers: String = signed_headers
    .iter()
    .filter_map(|name| headers.get(name).map(|val| format!("{name}:{val}")))
    .collect::<Vec<_>>()
    .join("\n");

  let signed_headers_joined = signed_headers.join(";");

  format!(
    "{method}\n{path}\n{query}\n{sorted_headers}\n{signed_headers_joined}\n{body_hash}"
  )
}

/// Builds the string to sign.
pub fn build_string_to_sign(
  timestamp: i64,
  nonce: &str,
  canonical_request_hash: &str,
) -> String {
  format!("{ALGORITHM}\n{timestamp}\n{nonce}\n{canonical_request_hash}")
}

/// Verifies the HMAC-SHA256 signature.
///
/// 1. Derives signing_key = HMAC-SHA256(secret, key_id)
/// 2. Computes expected = HMAC-SHA256(signing_key, string_to_sign)
/// 3. Compares in constant time
pub fn verify_signature(
  secret: &str,
  key_id: &str,
  timestamp: i64,
  nonce: &str,
  canonical_request_hash: &str,
  provided_signature: &str,
) -> Result<(), &'static str> {
  let signing_key =
    crypto::hmac_sha256_raw(secret.as_bytes(), key_id.as_bytes());
  let string_to_sign =
    build_string_to_sign(timestamp, nonce, canonical_request_hash);
  let expected =
    crypto::hmac_sha256_raw(&signing_key, string_to_sign.as_bytes());

  let expected_hex = hex::encode(&expected);
  constant_time_eq(&expected_hex, provided_signature)
    .then_some(())
    .ok_or("invalid signature")
}

/// Constant-time comparison.
fn constant_time_eq(a: &str, b: &str) -> bool {
  if a.len() != b.len() {
    return false;
  }
  a.as_bytes()
    .iter()
    .zip(b.as_bytes().iter())
    .fold(0, |acc, (x, y)| acc | (x ^ y))
    == 0
}
