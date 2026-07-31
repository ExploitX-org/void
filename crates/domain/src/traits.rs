use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::enums::{
  PaymentProvider, PaymentStatus, RegistrationType, TeamState,
  VerificationTarget,
};
use crate::primitives::{
  Email, MobileCc, MobileNumber, RegistrationInput, TeamLookup, TeamName,
};

#[derive(Debug, Clone)]
pub struct PaymentRow {
  pub id: Uuid,
  pub provider: String,
  pub provider_order_id: String,
  pub amount: i32,
  pub status: String,
  pub payment_link: String,
  pub ordered_at: DateTime<Utc>,
  pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
  #[error("not found: {0}")]
  NotFound(String),

  #[error("conflict: {0}")]
  Conflict(String),

  #[error("database error: {0}")]
  Database(String),
}

#[derive(Debug, Clone)]
pub struct TeamDetails {
  pub id: Uuid,
  pub registration_type: RegistrationType,
  pub team_name: String,
  pub state: TeamState,
  pub leader_name: String,
  pub leader_gender: String,
  pub leader_email: Email,
  pub leader_mobile_cc: String,
  pub leader_mobile_number: String,
  pub leader_location: String,
  pub member_name: Option<String>,
  pub member_gender: Option<String>,
  pub member_email: Option<Email>,
  pub member_mobile_cc: Option<String>,
  pub member_mobile_number: Option<String>,
  pub base_price_paise: Option<i32>,
  pub final_price_paise: Option<i32>,
  pub discount_code: Option<String>,
  pub resend_attempts: i32,
}

pub trait TeamRepository: Send + Sync {
  async fn create_team(
    &self,
    input: &RegistrationInput,
  ) -> std::result::Result<Uuid, RepositoryError>;

  async fn get_team_state(
    &self,
    team_id: Uuid,
  ) -> std::result::Result<TeamState, RepositoryError>;

  async fn set_team_pricing(
    &self,
    team_id: Uuid,
    base_price_paise: i32,
    final_price_paise: i32,
    discount_code: Option<&str>,
  ) -> std::result::Result<(), RepositoryError>;

  /// Returns `true` if the state was actually changed, `false` if already at target.
  async fn update_team_state(
    &self,
    team_id: Uuid,
    state: TeamState,
  ) -> std::result::Result<bool, RepositoryError>;

  async fn find_by_email(
    &self,
    email: &Email,
  ) -> std::result::Result<Option<Uuid>, RepositoryError>;

  async fn find_team_id_by_email(
    &self,
    email: &Email,
  ) -> std::result::Result<Uuid, RepositoryError>;

  async fn find_by_id(
    &self,
    team_id: Uuid,
  ) -> std::result::Result<TeamDetails, RepositoryError>;

  async fn check_team_name_taken(
    &self,
    team_name: &TeamName,
  ) -> std::result::Result<bool, RepositoryError>;

  async fn find_by_mobile(
    &self,
    cc: &MobileCc,
    number: &MobileNumber,
  ) -> std::result::Result<Option<Uuid>, RepositoryError>;

  async fn find_team_by_email(
    &self,
    email: &Email,
  ) -> std::result::Result<Option<TeamLookup>, RepositoryError>;

  async fn find_team_by_mobile(
    &self,
    cc: &MobileCc,
    number: &MobileNumber,
  ) -> std::result::Result<Option<TeamLookup>, RepositoryError>;

  async fn find_team_by_name(
    &self,
    team_name: &TeamName,
  ) -> std::result::Result<Option<TeamLookup>, RepositoryError>;

  async fn is_registration_open(
    &self,
  ) -> std::result::Result<bool, RepositoryError>;

  async fn increment_resend_attempts(
    &self,
    team_id: Uuid,
  ) -> std::result::Result<i32, RepositoryError>;

  async fn delete_team(
    &self,
    team_id: Uuid,
  ) -> std::result::Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct OtpStoredData {
  pub otp_hash: Option<String>,
  pub expires_at: Option<DateTime<Utc>>,
  pub attempts: i32,
}

pub trait VerificationRepository: Send + Sync {
  async fn get_otp_data(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
  ) -> std::result::Result<Option<OtpStoredData>, RepositoryError>;

  async fn store_otp(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
    email: &Email,
    otp_hash: &str,
    expires_at: DateTime<Utc>,
  ) -> std::result::Result<(), RepositoryError>;

  async fn verify_otp(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
    otp_hash: &str,
  ) -> std::result::Result<bool, RepositoryError>;

  async fn is_verified(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
  ) -> std::result::Result<bool, RepositoryError>;

  async fn increment_otp_attempts(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
  ) -> std::result::Result<i32, RepositoryError>;

  async fn clear_otp(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
  ) -> std::result::Result<(), RepositoryError>;
}

pub trait PaymentRepository: Send + Sync {
  async fn create_payment(
    &self,
    team_id: Uuid,
    provider: PaymentProvider,
    provider_order_id: &str,
    amount: i32,
    payment_link: &str,
  ) -> std::result::Result<Uuid, RepositoryError>;

  async fn update_payment_status(
    &self,
    payment_id: Uuid,
    status: PaymentStatus,
  ) -> std::result::Result<(), RepositoryError>;

  async fn get_payment_amount(
    &self,
    team_id: Uuid,
  ) -> std::result::Result<i32, RepositoryError>;

  async fn find_payment_by_team_id(
    &self,
    team_id: Uuid,
  ) -> std::result::Result<Option<PaymentRow>, RepositoryError>;

  async fn next_invoice_number(
    &self,
  ) -> std::result::Result<i32, RepositoryError>;
}
