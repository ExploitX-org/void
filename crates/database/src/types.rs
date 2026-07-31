use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LeaderRow {
  pub id: Uuid,
  pub team_id: Uuid,
  pub name: String,
  pub gender: String,
  pub email: String,
  pub mobile_cc: String,
  pub mobile_number: String,
  pub college_name: String,
  pub degree: String,
  pub department: String,
  pub year_of_study: i32,
  pub location: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemberRow {
  pub id: Uuid,
  pub team_id: Uuid,
  pub name: String,
  pub gender: String,
  pub email: String,
  pub mobile_cc: String,
  pub mobile_number: String,
  pub college_name: String,
  pub degree: String,
  pub department: String,
  pub year_of_study: i32,
  pub location: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VerificationRow {
  pub id: Uuid,
  pub team_id: Uuid,
  pub target: String,
  pub email: String,
  pub otp_hash: Option<String>,
  pub otp_expires_at: Option<DateTime<Utc>>,
  pub otp_attempts: i32,
  pub verified: bool,
  pub verified_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
}
