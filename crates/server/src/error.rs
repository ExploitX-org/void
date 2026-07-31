use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use domain::ParseError;
use domain::RepositoryError;
use payment::PaymentError;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorResponse {
  #[schema(example = "error")]
  pub r#type: String,
  pub status_code: u16,
  pub msg: String,
  pub error: ApiErrorDetail,
  pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorDetail {
  #[schema(example = "bad_request")]
  pub code: String,
  #[schema(example = "invalid input")]
  pub msg: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
  #[error("bad request: {0}")]
  BadRequest(String),

  #[error("unauthorized: {0}")]
  Unauthorized(String),

  #[error("not found: {0}")]
  NotFound(String),

  #[error("conflict: {0}")]
  Conflict(String),

  #[error("too many requests: {0}")]
  TooManyRequests(String),

  #[error("internal: {0}")]
  Internal(String),
}

impl From<ParseError> for ApiError {
  fn from(e: ParseError) -> Self {
    match e {
      ParseError::Internal(msg) => Self::Internal(msg),
      other => Self::BadRequest(other.to_string()),
    }
  }
}

impl From<RepositoryError> for ApiError {
  fn from(e: RepositoryError) -> Self {
    match &e {
      RepositoryError::NotFound(_) => Self::NotFound(e.to_string()),
      RepositoryError::Conflict(_) => Self::Conflict(e.to_string()),
      RepositoryError::Database(_) => Self::Internal(e.to_string()),
    }
  }
}

impl From<PaymentError> for ApiError {
  fn from(e: PaymentError) -> Self {
    match &e {
      PaymentError::Gateway(msg) => {
        tracing::error!("payment gateway error: {msg}");
        Self::Internal("payment gateway error".into())
      }
      PaymentError::InvalidSignature => {
        Self::Unauthorized("invalid payment signature".into())
      }
      PaymentError::Network(msg) => Self::Internal(msg.clone()),
    }
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let (status, code, msg, detail) = match &self {
      Self::BadRequest(msg) => (
        StatusCode::BAD_REQUEST,
        "bad_request",
        msg.clone(),
        msg.clone(),
      ),
      Self::Unauthorized(msg) => (
        StatusCode::UNAUTHORIZED,
        "unauthorized",
        msg.clone(),
        msg.clone(),
      ),
      Self::NotFound(msg) => {
        (StatusCode::NOT_FOUND, "not_found", msg.clone(), msg.clone())
      }
      Self::TooManyRequests(msg) => (
        StatusCode::TOO_MANY_REQUESTS,
        "too_many_requests",
        msg.clone(),
        msg.clone(),
      ),
      Self::Conflict(msg) => {
        (StatusCode::CONFLICT, "conflict", msg.clone(), msg.clone())
      }
      Self::Internal(msg) => {
        tracing::error!("{msg}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "internal_error",
          "An internal error occurred.".into(),
          String::new(),
        )
      }
    };

    let body = serde_json::json!({
        "type": "error",
        "status_code": status.as_u16(),
        "msg": msg,
        "error": {
            "code": code,
            "msg": detail,
        },
        "data": null,
    });

    (status, Json(body)).into_response()
  }
}
