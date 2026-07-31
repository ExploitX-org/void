use axum::Json;
use axum::extract::State;

use crate::AppState;
use crate::error::ApiError;
use crate::services::verification::VerificationService;
use crate::types::{
  ApiSuccessResponse, EmptyData, ParsedResendRequest, ParsedVerifyRequest,
  ResendRequest, VerifyRequest,
};

#[utoipa::path(
    post,
    path = crate::routes::constants::VERIFY,
    tag = "public",
    request_body = VerifyRequest,
    responses(
        (status = 200, description = "OTP verified successfully", body = ApiSuccessResponse<EmptyData>),
        (status = 400, description = "Invalid OTP or request", body = crate::error::ApiErrorResponse),
        (status = 404, description = "Team not found", body = crate::error::ApiErrorResponse),
        (status = 409, description = "Team not in verification state", body = crate::error::ApiErrorResponse),
        (status = 429, description = "Too many attempts", body = crate::error::ApiErrorResponse),
    ),
)]
pub async fn verify(
  State(state): State<AppState>,
  Json(req): Json<VerifyRequest>,
) -> Result<ApiSuccessResponse<EmptyData>, ApiError> {
  let parsed = ParsedVerifyRequest {
    team_id: req
      .team_id
      .parse()
      .map_err(|e| ApiError::BadRequest(format!("invalid team_id: {e}")))?,
    leader_otp: req.otp.leader.try_into()?,
    member_otp: req.otp.member.map(|s| s.try_into()).transpose()?,
  };

  VerificationService::verify(&state, parsed).await?;

  Ok(ApiSuccessResponse::<EmptyData>::message(
    "OTP verified successfully.",
  ))
}

#[utoipa::path(
    post,
    path = crate::routes::constants::RESEND,
    tag = "public",
    request_body = ResendRequest,
    responses(
        (status = 200, description = "OTP resent", body = ApiSuccessResponse<EmptyData>),
        (status = 400, description = "Bad request", body = crate::error::ApiErrorResponse),
        (status = 404, description = "Team not found", body = crate::error::ApiErrorResponse),
        (status = 409, description = "Team not in verification state / deleted due to excess resend attempts", body = crate::error::ApiErrorResponse),
    ),
)]
pub async fn resend(
  State(state): State<AppState>,
  Json(req): Json<ResendRequest>,
) -> Result<ApiSuccessResponse<EmptyData>, ApiError> {
  let parsed = ParsedResendRequest {
    team_id: req
      .team_id
      .parse()
      .map_err(|e| ApiError::BadRequest(format!("invalid team_id: {e}")))?,
  };

  VerificationService::resend_otp(&state, parsed).await?;

  Ok(ApiSuccessResponse::<EmptyData>::message(
    "OTP has been resent.",
  ))
}
