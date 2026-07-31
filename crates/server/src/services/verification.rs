use chrono::Utc;
use domain::{
  OtpCode, TeamRepository, TeamState, VerificationRepository,
  VerificationTarget,
};

use crate::AppState;
use crate::error::ApiError;
use crate::types::{ParsedResendRequest, ParsedVerifyRequest};

pub struct VerificationService;

impl VerificationService {
  pub async fn verify(
    state: &AppState,
    req: ParsedVerifyRequest,
  ) -> Result<(), ApiError> {
    let team = state.team_repo.find_by_id(req.team_id).await?;

    match team.state {
      TeamState::PendingVerification | TeamState::Verified => {}
      _ => {
        return Err(ApiError::Conflict(
          "team is not in verification state".into(),
        ));
      }
    }

    if team.member_email.is_none() {
      Self::verify_single(state, req).await
    } else {
      Self::verify_couple(state, req).await
    }
  }

  async fn verify_single(
    state: &AppState,
    req: ParsedVerifyRequest,
  ) -> Result<(), ApiError> {
    let max_attempts = state.config.otp_max_attempts as i32;

    let data = state
      .verification_repo
      .get_otp_data(req.team_id, VerificationTarget::Leader)
      .await?;

    let no_otp = data.is_none();
    let mut expired = false;
    let mut locked = false;
    let mut invalid = false;
    let otp_hash =
      crate::crypto::sha256_hex(req.leader_otp.as_str().as_bytes());

    if let Some(ref data) = data {
      if data.attempts >= max_attempts {
        locked = true;
      } else if data.expires_at.is_some_and(|exp| Utc::now() > exp) {
        expired = true;
      } else if data.otp_hash.as_deref() != Some(otp_hash.as_str()) {
        invalid = true;
      }
    }

    if locked {
      return Err(ApiError::TooManyRequests("too many OTP attempts".into()));
    }
    if no_otp {
      return Err(ApiError::BadRequest("no OTP has been requested".into()));
    }
    if expired {
      return Err(ApiError::BadRequest(
        "OTP has expired. please request a new one".into(),
      ));
    }
    if invalid {
      let new_attempts = state
        .verification_repo
        .increment_otp_attempts(req.team_id, VerificationTarget::Leader)
        .await?;
      if new_attempts >= max_attempts {
        return Err(ApiError::TooManyRequests("too many OTP attempts".into()));
      }
      return Err(ApiError::BadRequest("invalid OTP".into()));
    }

    let verified = state
      .verification_repo
      .verify_otp(req.team_id, VerificationTarget::Leader, &otp_hash)
      .await?;

    if !verified {
      return Err(ApiError::BadRequest("failed to verify OTP".into()));
    }

    state
      .team_repo
      .update_team_state(req.team_id, TeamState::Verified)
      .await?;

    Ok(())
  }

  async fn verify_couple(
    state: &AppState,
    req: ParsedVerifyRequest,
  ) -> Result<(), ApiError> {
    let member_otp = req.member_otp.ok_or_else(|| {
      ApiError::BadRequest("member_otp is required for a couple team".into())
    })?;

    let leader_hash =
      crate::crypto::sha256_hex(req.leader_otp.as_str().as_bytes());
    let member_hash = crate::crypto::sha256_hex(member_otp.as_str().as_bytes());

    let max_attempts = state.config.otp_max_attempts as i32;

    let leader_data = state
      .verification_repo
      .get_otp_data(req.team_id, VerificationTarget::Leader)
      .await?;

    let member_data = state
      .verification_repo
      .get_otp_data(req.team_id, VerificationTarget::Member)
      .await?;

    let leader_no_otp = leader_data.is_none();
    let member_no_otp = member_data.is_none();

    let mut leader_expired = false;
    let mut leader_locked = false;
    let mut leader_invalid = false;

    if let Some(ref data) = leader_data {
      if data.attempts >= max_attempts {
        leader_locked = true;
      } else if data.expires_at.is_some_and(|exp| Utc::now() > exp) {
        leader_expired = true;
      } else if data.otp_hash.as_deref() != Some(leader_hash.as_str()) {
        leader_invalid = true;
      }
    }

    let mut member_expired = false;
    let mut member_locked = false;
    let mut member_invalid = false;

    if let Some(ref data) = member_data {
      if data.attempts >= max_attempts {
        member_locked = true;
      } else if data.expires_at.is_some_and(|exp| Utc::now() > exp) {
        member_expired = true;
      } else if data.otp_hash.as_deref() != Some(member_hash.as_str()) {
        member_invalid = true;
      }
    }

    if leader_locked {
      return Err(ApiError::TooManyRequests("too many OTP attempts".into()));
    }
    if member_locked {
      return Err(ApiError::TooManyRequests("too many OTP attempts".into()));
    }

    if leader_no_otp {
      return Err(ApiError::BadRequest(
        "no leader OTP has been requested".into(),
      ));
    }
    if member_no_otp {
      return Err(ApiError::BadRequest(
        "no member OTP has been requested".into(),
      ));
    }

    if leader_expired {
      return Err(ApiError::BadRequest(
        "leader OTP has expired. please request a new one".into(),
      ));
    }
    if member_expired {
      return Err(ApiError::BadRequest(
        "member OTP has expired. please request a new one".into(),
      ));
    }

    if leader_invalid && member_invalid {
      let leader_attempts = state
        .verification_repo
        .increment_otp_attempts(req.team_id, VerificationTarget::Leader)
        .await?;
      let member_attempts = state
        .verification_repo
        .increment_otp_attempts(req.team_id, VerificationTarget::Member)
        .await?;
      if leader_attempts >= max_attempts {
        return Err(ApiError::TooManyRequests("too many OTP attempts".into()));
      }
      if member_attempts >= max_attempts {
        return Err(ApiError::TooManyRequests("too many OTP attempts".into()));
      }
      return Err(ApiError::BadRequest("both OTPs are invalid".into()));
    }

    if leader_invalid {
      let leader_attempts = state
        .verification_repo
        .increment_otp_attempts(req.team_id, VerificationTarget::Leader)
        .await?;
      if leader_attempts >= max_attempts {
        return Err(ApiError::TooManyRequests("too many OTP attempts".into()));
      }
      return Err(ApiError::BadRequest("leader OTP is invalid".into()));
    }

    if member_invalid {
      let member_attempts = state
        .verification_repo
        .increment_otp_attempts(req.team_id, VerificationTarget::Member)
        .await?;
      if member_attempts >= max_attempts {
        return Err(ApiError::TooManyRequests("too many OTP attempts".into()));
      }
      return Err(ApiError::BadRequest("member OTP is invalid".into()));
    }

    let leader_verified = state
      .verification_repo
      .is_verified(req.team_id, VerificationTarget::Leader)
      .await?;

    let member_verified = state
      .verification_repo
      .is_verified(req.team_id, VerificationTarget::Member)
      .await?;

    if !leader_verified
      && !state
        .verification_repo
        .verify_otp(req.team_id, VerificationTarget::Leader, &leader_hash)
        .await?
    {
      return Err(ApiError::BadRequest(
        "failed to mark leader verified".into(),
      ));
    }

    if !member_verified
      && !state
        .verification_repo
        .verify_otp(req.team_id, VerificationTarget::Member, &member_hash)
        .await?
    {
      return Err(ApiError::BadRequest(
        "failed to mark member verified".into(),
      ));
    }

    state
      .team_repo
      .update_team_state(req.team_id, TeamState::Verified)
      .await?;

    Ok(())
  }

  pub async fn resend_otp(
    state: &AppState,
    req: ParsedResendRequest,
  ) -> Result<(), ApiError> {
    let team = state.team_repo.find_by_id(req.team_id).await?;

    match team.state {
      TeamState::PendingVerification | TeamState::Verified => {}
      _ => {
        return Err(ApiError::Conflict(
          "team is not in verification state".into(),
        ));
      }
    }

    if team.resend_attempts >= state.config.max_resend_otp_retry_attempts as i32
    {
      state.team_repo.delete_team(req.team_id).await?;
      return Err(ApiError::Conflict(
        "Team deleted due to too many resend attempts. Please register again."
          .into(),
      ));
    }

    let expires_at =
      Utc::now() + chrono::Duration::seconds(state.config.otp_ttl_secs);

    let leader_otp = OtpCode::generate()?;
    let leader_hash = crate::crypto::sha256_hex(leader_otp.as_str().as_bytes());

    state
      .verification_repo
      .store_otp(
        req.team_id,
        VerificationTarget::Leader,
        &team.leader_email,
        &leader_hash,
        expires_at,
      )
      .await?;

    state
      .email_client
      .send_otp(&team.leader_email, &leader_otp, state.config.otp_ttl_secs)
      .await
      .map_err(|e| {
        ApiError::Internal(format!("failed to send otp email to leader: {e}"))
      })?;

    if let Some(ref member_email) = team.member_email {
      let member_otp = OtpCode::generate()?;
      let member_hash =
        crate::crypto::sha256_hex(member_otp.as_str().as_bytes());

      state
        .verification_repo
        .store_otp(
          req.team_id,
          VerificationTarget::Member,
          member_email,
          &member_hash,
          expires_at,
        )
        .await?;

      state
        .email_client
        .send_otp(member_email, &member_otp, state.config.otp_ttl_secs)
        .await
        .map_err(|e| {
          ApiError::Internal(format!("failed to send otp email to member: {e}"))
        })?;
    }

    state
      .team_repo
      .increment_resend_attempts(req.team_id)
      .await?;

    Ok(())
  }
}
