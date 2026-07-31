use axum::Json;
use axum::extract::State;

use crate::AppState;
use crate::error::ApiError;
use crate::services::payment::PaymentService;
use crate::types::{
  ApiSuccessResponse, CreatePaymentData, CreatePaymentRequest,
  ParsedCreatePaymentRequest, ParsedPaymentStatusRequest, PaymentStatusData,
  PaymentStatusRequest,
};

#[utoipa::path(
    post,
    path = crate::routes::constants::CREATE_PAYMENT,
    tag = "public",
    request_body = CreatePaymentRequest,
    responses(
        (status = 200, description = "Payment session created", body = ApiSuccessResponse<CreatePaymentData>),
        (status = 400, description = "Email not verified or payment error", body = crate::error::ApiErrorResponse),
        (status = 404, description = "Team not found", body = crate::error::ApiErrorResponse),
    ),
)]
pub async fn create_payment(
  State(state): State<AppState>,
  Json(req): Json<CreatePaymentRequest>,
) -> Result<ApiSuccessResponse<CreatePaymentData>, ApiError> {
  let parsed = ParsedCreatePaymentRequest {
    team_id: req
      .team_id
      .parse()
      .map_err(|e| ApiError::BadRequest(format!("invalid team_id: {e}")))?,
  };

  let payment_link = PaymentService::create(&state, parsed).await?;

  Ok(ApiSuccessResponse::new(
    "Payment session created",
    CreatePaymentData { payment_link },
  ))
}

#[utoipa::path(
    post,
    path = crate::routes::constants::PAYMENT_STATUS,
    tag = "public",
    request_body = PaymentStatusRequest,
    responses(
        (status = 200, description = "Payment status retrieved", body = ApiSuccessResponse<PaymentStatusData>),
        (status = 404, description = "Team or payment not found", body = crate::error::ApiErrorResponse),
    ),
)]
pub async fn payment_status(
  State(state): State<AppState>,
  Json(req): Json<PaymentStatusRequest>,
) -> Result<ApiSuccessResponse<PaymentStatusData>, ApiError> {
  let parsed = ParsedPaymentStatusRequest {
    team_id: req
      .team_id
      .parse()
      .map_err(|e| ApiError::BadRequest(format!("invalid team_id: {e}")))?,
  };

  let data = PaymentService::status(&state, parsed).await?;

  Ok(ApiSuccessResponse::new("Payment status retrieved", data))
}
