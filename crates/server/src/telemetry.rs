use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;
use uuid::Uuid;

pub async fn log_request(req: Request<Body>, next: Next) -> Response {
  let start = Instant::now();
  let method = req.method().clone();
  let path = req.uri().path().to_string();
  let request_id = Uuid::new_v4().to_string();
  let client_ip = req
    .headers()
    .get("x-forwarded-for")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("unknown")
    .to_string();

  let response = next.run(req).await;

  let status = response.status().as_u16();
  let latency_ms = start.elapsed().as_millis();

  match status {
    500.. => tracing::error!(
      target: "void_srv::request",
      method = %method,
      path = %path,
      status = status,
      latency_ms = latency_ms,
      request_id = %request_id,
      client_ip = %client_ip,
      "request failed",
    ),
    400.. => tracing::warn!(
      target: "void_srv::request",
      method = %method,
      path = %path,
      status = status,
      latency_ms = latency_ms,
      request_id = %request_id,
      client_ip = %client_ip,
      "request rejected",
    ),
    _ => tracing::info!(
      target: "void_srv::request",
      method = %method,
      path = %path,
      status = status,
      latency_ms = latency_ms,
      request_id = %request_id,
      client_ip = %client_ip,
      "request",
    ),
  }

  response
}
