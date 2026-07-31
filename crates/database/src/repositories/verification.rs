use chrono::{DateTime, Utc};
use domain::{
  Email, RepositoryError, VerificationRepository, VerificationTarget,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgVerificationRepository {
  pool: PgPool,
}

impl PgVerificationRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

impl VerificationRepository for PgVerificationRepository {
  async fn get_otp_data(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
  ) -> Result<Option<domain::OtpStoredData>, RepositoryError> {
    let target_str = match target {
      VerificationTarget::Leader => "leader",
      VerificationTarget::Member => "member",
    };

    let row: Option<(Option<String>, Option<DateTime<Utc>>, i32)> =
      sqlx::query_as(
        r#"
            SELECT otp_hash, otp_expires_at, otp_attempts
            FROM verifications
            WHERE team_id = $1 AND target = $2::verification_target
            "#,
      )
      .bind(team_id)
      .bind(target_str)
      .fetch_optional(&self.pool)
      .await
      .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(row.map(|(hash, exp, att)| domain::OtpStoredData {
      otp_hash: hash,
      expires_at: exp,
      attempts: att,
    }))
  }

  async fn store_otp(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
    email: &Email,
    otp_hash: &str,
    expires_at: DateTime<Utc>,
  ) -> Result<(), RepositoryError> {
    let target_str = match target {
      VerificationTarget::Leader => "leader",
      VerificationTarget::Member => "member",
    };

    sqlx::query(
            r#"
            INSERT INTO verifications (team_id, target, email, otp_hash, otp_expires_at, otp_attempts)
            VALUES ($1, $2::verification_target, $3, $4, $5, 0)
            ON CONFLICT (team_id, target)
            DO UPDATE SET
                email = $3,
                otp_hash = $4,
                otp_expires_at = $5,
                otp_attempts = 0,
                verified = FALSE,
                verified_at = NULL
            "#,
        )
        .bind(team_id)
        .bind(target_str)
        .bind(email.as_str())
        .bind(otp_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(())
  }

  async fn verify_otp(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
    otp_hash: &str,
  ) -> Result<bool, RepositoryError> {
    let target_str = match target {
      VerificationTarget::Leader => "leader",
      VerificationTarget::Member => "member",
    };

    let verified: Option<bool> = sqlx::query_scalar(
      r#"
          UPDATE verifications
          SET verified = TRUE, verified_at = NOW()
          WHERE team_id = $1
            AND target = $2::verification_target
            AND otp_hash = $3
            AND (otp_expires_at IS NULL OR otp_expires_at > NOW())
            AND verified = FALSE
          RETURNING verified
          "#,
    )
    .bind(team_id)
    .bind(target_str)
    .bind(otp_hash)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    match verified {
      Some(true) => Ok(true),
      _ => {
        sqlx::query(
          r#"
              UPDATE verifications
              SET otp_attempts = otp_attempts + 1
              WHERE team_id = $1 AND target = $2::verification_target
              "#,
        )
        .bind(team_id)
        .bind(target_str)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(false)
      }
    }
  }

  async fn is_verified(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
  ) -> Result<bool, RepositoryError> {
    let target_str = match target {
      VerificationTarget::Leader => "leader",
      VerificationTarget::Member => "member",
    };

    let verified: Option<bool> = sqlx::query_scalar(
      r#"
            SELECT verified FROM verifications
            WHERE team_id = $1 AND target = $2::verification_target
            "#,
    )
    .bind(team_id)
    .bind(target_str)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(verified.unwrap_or(false))
  }

  async fn increment_otp_attempts(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
  ) -> Result<i32, RepositoryError> {
    let target_str = match target {
      VerificationTarget::Leader => "leader",
      VerificationTarget::Member => "member",
    };

    let attempts: i32 = sqlx::query_scalar(
      r#"
            UPDATE verifications
            SET otp_attempts = otp_attempts + 1
            WHERE team_id = $1 AND target = $2::verification_target
            RETURNING otp_attempts
            "#,
    )
    .bind(team_id)
    .bind(target_str)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(attempts)
  }

  async fn clear_otp(
    &self,
    team_id: Uuid,
    target: VerificationTarget,
  ) -> Result<(), RepositoryError> {
    let target_str = match target {
      VerificationTarget::Leader => "leader",
      VerificationTarget::Member => "member",
    };

    sqlx::query(
      r#"
            UPDATE verifications
            SET otp_hash = NULL, otp_expires_at = NULL, otp_attempts = 0,
                verified = FALSE, verified_at = NULL
            WHERE team_id = $1 AND target = $2::verification_target
            "#,
    )
    .bind(team_id)
    .bind(target_str)
    .execute(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(())
  }
}
