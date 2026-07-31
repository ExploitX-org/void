use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use domain::OtpCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PersonInfo {
  #[schema(example = "Alice")]
  pub name: String,
  #[schema(example = "female")]
  pub gender: String,
  #[schema(example = "alice@citchennai.net")]
  pub email: String,
  pub mobile: MobileInfo,
  #[schema(example = "Chennai Institute of Technology")]
  pub college: String,
  #[schema(example = "B.Tech")]
  pub degree: String,
  #[schema(example = "CSE")]
  pub department: String,
  #[schema(example = 3)]
  pub year: i32,
  #[schema(example = "Chennai")]
  pub location: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MobileInfo {
  #[schema(example = "+91")]
  pub cc: String,
  #[schema(example = "9876543210")]
  pub number: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
  #[schema(example = "single")]
  pub registration_type: String,
  #[schema(example = "AlphaTeam")]
  pub team_name: String,
  pub leader: PersonInfo,
  pub member: Option<PersonInfo>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OtpPair {
  #[schema(example = "A3X9K2M7")]
  pub leader: String,
  #[schema(example = "B4Y0L3N8")]
  pub member: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyRequest {
  #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
  pub team_id: String,
  pub otp: OtpPair,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResendRequest {
  #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
  pub team_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePaymentRequest {
  #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
  pub team_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaymentStatusRequest {
  #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
  pub team_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterData {
  #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
  pub team_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmptyData;

#[derive(Debug, Serialize, ToSchema)]
pub struct CreatePaymentData {
  #[schema(
    example = "https://payments.cashfree.com/orderpay/session_Lx09qR3p"
  )]
  pub payment_link: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentStatusData {
  #[schema(example = "completed")]
  pub status: String,
  #[schema(example = "cashfree")]
  pub payment_provider: Option<String>,
  #[schema(example = "order_xyz789")]
  pub payment_order_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiSuccessResponse<T: Serialize> {
  #[schema(example = "success")]
  pub r#type: String,
  #[schema(example = 200)]
  pub status_code: u16,
  #[schema(example = "ok")]
  pub msg: String,
  pub error: Option<serde_json::Value>,
  pub data: Option<T>,
}

impl<T: Serialize> ApiSuccessResponse<T> {
  pub fn new(msg: impl Into<String>, data: T) -> Self {
    Self {
      r#type: "success".into(),
      status_code: 200,
      msg: msg.into(),
      error: None,
      data: Some(data),
    }
  }

  pub fn with_status(mut self, code: u16) -> Self {
    self.status_code = code;
    self
  }

  pub fn message(msg: impl Into<String>) -> Self {
    Self {
      r#type: "success".into(),
      status_code: 200,
      msg: msg.into(),
      error: None,
      data: None,
    }
  }
}

impl<T: Serialize> IntoResponse for ApiSuccessResponse<T> {
  fn into_response(self) -> Response {
    let status = StatusCode::from_u16(self.status_code)
      .expect("ApiSuccessResponse has an invalid status_code");
    (status, Json(self)).into_response()
  }
}

pub struct ParsedVerifyRequest {
  pub team_id: uuid::Uuid,
  pub leader_otp: OtpCode,
  pub member_otp: Option<OtpCode>,
}

pub struct ParsedResendRequest {
  pub team_id: uuid::Uuid,
}

pub struct ParsedCreatePaymentRequest {
  pub team_id: uuid::Uuid,
}

pub struct ParsedPaymentStatusRequest {
  pub team_id: uuid::Uuid,
}
