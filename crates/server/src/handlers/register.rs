use axum::Json;
use axum::extract::State;

use domain::{
  CollegeName, Degree, Department, Email, Gender, Location, MobileCc,
  MobileNumber, Name, PersonData, RegistrationInput, RegistrationType,
  TeamName, TeamRepository, YearOfStudy,
};

use crate::AppState;
use crate::config::Config;
use crate::error::ApiError;
use crate::services::registration::{RegisterAction, RegistrationService};
use crate::types::{
  ApiSuccessResponse, PersonInfo, RegisterData, RegisterRequest,
};

#[utoipa::path(
    post,
    path = crate::routes::constants::REGISTER,
    tag = "public",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration initiated", body = ApiSuccessResponse<RegisterData>),
        (status = 200, description = "OTP resent for existing pending team", body = ApiSuccessResponse<RegisterData>),
        (status = 400, description = "Bad request", body = crate::error::ApiErrorResponse),
        (status = 409, description = "Conflict", body = crate::error::ApiErrorResponse),
    ),
)]
pub async fn register(
  State(state): State<AppState>,
  Json(req): Json<RegisterRequest>,
) -> Result<ApiSuccessResponse<RegisterData>, ApiError> {
  if !state.team_repo.is_registration_open().await? {
    return Err(ApiError::BadRequest("registration is closed".into()));
  }

  let reg_type: RegistrationType = req.registration_type.try_into()?;

  let team_name = TeamName::parse_with_len(
    &req.team_name,
    state.config.team_name_min_len,
    state.config.team_name_max_len,
  )?;

  let leader = parse_person_info(req.leader, &state.config)?;

  let member = match req.member {
    Some(m) => Some(parse_person_info(m, &state.config)?),
    None => None,
  };

  if team_name.as_str() == leader.email.as_str() {
    return Err(ApiError::BadRequest(
      "Team name cannot be same as email".into(),
    ));
  }
  if team_name.as_str() == leader.mobile_number.as_str() {
    return Err(ApiError::BadRequest(
      "Team name cannot be same as mobile number".into(),
    ));
  }
  if leader.email.as_str() == leader.mobile_number.as_str() {
    return Err(ApiError::BadRequest(
      "Email cannot be same as mobile number".into(),
    ));
  }

  match (reg_type, &member) {
    (RegistrationType::Single, Some(_)) => {
      return Err(ApiError::BadRequest(
        "member must be null for single registration".into(),
      ));
    }
    (RegistrationType::Couple, None) => {
      return Err(ApiError::BadRequest(
        "member is required for couple registration".into(),
      ));
    }
    (RegistrationType::Couple, Some(m)) => {
      if leader.email == m.email {
        return Err(ApiError::BadRequest(
          "Leader and member must have different email addresses".into(),
        ));
      }
      if leader.mobile_cc == m.mobile_cc
        && leader.mobile_number == m.mobile_number
      {
        return Err(ApiError::BadRequest(
          "Leader and member must have different mobile numbers".into(),
        ));
      }
      if leader.email.as_str() == m.mobile_number.as_str() {
        return Err(ApiError::BadRequest(
          "Leader email cannot be same as member mobile".into(),
        ));
      }
      if leader.mobile_number.as_str() == m.email.as_str() {
        return Err(ApiError::BadRequest(
          "Leader mobile cannot be same as member email".into(),
        ));
      }
      if team_name.as_str() == m.email.as_str() {
        return Err(ApiError::BadRequest(
          "Team name cannot be same as member email".into(),
        ));
      }
      if team_name.as_str() == m.mobile_number.as_str() {
        return Err(ApiError::BadRequest(
          "Team name cannot be same as member mobile".into(),
        ));
      }
    }
    _ => {}
  }

  let input = match reg_type {
    RegistrationType::Single => RegistrationInput::Single { team_name, leader },
    RegistrationType::Couple => RegistrationInput::Couple {
      team_name,
      leader,
      member: Box::new(member.ok_or_else(|| {
        ApiError::BadRequest(
          "member is required for couple registration".into(),
        )
      })?),
    },
  };

  let action = RegistrationService::register(
    &state,
    input,
    state.config.portal_start,
    state.config.base_price_paise,
    state.config.otp_ttl_secs,
  )
  .await?;

  match action {
    RegisterAction::Created { team_id } => Ok(
      ApiSuccessResponse::new(
        "Registration initiated. Check your email for OTP.",
        RegisterData { team_id },
      )
      .with_status(201),
    ),
    RegisterAction::Resent { team_id } => Ok(
      ApiSuccessResponse::new(
        "OTP has been resent. Check your email.",
        RegisterData { team_id },
      )
      .with_status(200),
    ),
  }
}

fn parse_person_info(
  p: PersonInfo,
  config: &Config,
) -> Result<PersonData, ApiError> {
  let name: Name = p.name.try_into()?;
  let gender: Gender = p.gender.try_into()?;
  let email: Email = p.email.try_into()?;
  let mobile_cc = MobileCc::parse_with_len(
    p.mobile.cc,
    config.mobile_cc_min_len,
    config.mobile_cc_max_len,
  )?;
  let mobile_number = MobileNumber::parse_with_len(
    p.mobile.number,
    config.mobile_number_min_len,
    config.mobile_number_max_len,
  )?;
  let college_name: CollegeName = p.college.try_into()?;
  let degree: Degree = p.degree.try_into()?;
  let department: Department = p.department.try_into()?;
  let year_of_study: YearOfStudy = p.year.try_into()?;
  let location: Location = p.location.try_into()?;

  Ok(PersonData {
    name,
    gender,
    email,
    mobile_cc,
    mobile_number,
    college_name,
    degree,
    department,
    year_of_study,
    location,
  })
}
