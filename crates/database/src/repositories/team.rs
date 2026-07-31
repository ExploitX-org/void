use domain::{
  Email, MobileCc, MobileNumber, RegistrationInput, RegistrationType,
  RepositoryError, TeamDetails, TeamLookup, TeamName, TeamRepository,
  TeamState,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::types::{LeaderRow, MemberRow};

#[derive(Clone)]
pub struct PgTeamRepository {
  pool: PgPool,
}

impl PgTeamRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

impl TeamRepository for PgTeamRepository {
  async fn create_team(
    &self,
    input: &RegistrationInput,
  ) -> Result<Uuid, RepositoryError> {
    let reg_type_str = match input.registration_type() {
      RegistrationType::Single => "single",
      RegistrationType::Couple => "couple",
    };
    let team_name = input.team_name().as_str();
    let leader = input.leader();

    let mut tx = self
      .pool
      .begin()
      .await
      .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let team_id: Uuid = sqlx::query_scalar(
      r#"
            INSERT INTO teams (registration_type, team_name, state)
            VALUES ($1::registration_type, $2, 'pending_verification')
            RETURNING id
            "#,
    )
    .bind(reg_type_str)
    .bind(team_name)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
      if let sqlx::Error::Database(ref db_err) = e
        && db_err.code().as_deref() == Some("23505")
      {
        let msg = match db_err.constraint() {
          Some("idx_teams_name_lower") => "team name already exists",
          _ => "unique constraint violation",
        };
        return RepositoryError::Conflict(msg.into());
      }
      RepositoryError::Database(e.to_string())
    })?;

    sqlx::query(
      r#"
            INSERT INTO leaders (team_id, name, gender, email, mobile_cc, mobile_number,
                                 college_name, degree, department, year_of_study, location)
            VALUES ($1, $2, $3::gender, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
    )
    .bind(team_id)
    .bind(leader.name.as_str())
    .bind(match leader.gender {
      domain::Gender::Male => "male",
      domain::Gender::Female => "female",
      domain::Gender::Trans => "trans",
      domain::Gender::Others => "others",
    })
    .bind(leader.email.as_str())
    .bind(leader.mobile_cc.as_str())
    .bind(leader.mobile_number.as_str())
    .bind(leader.college_name.as_str())
    .bind(leader.degree.as_str())
    .bind(leader.department.as_str())
    .bind(leader.year_of_study.get() as i32)
    .bind(leader.location.as_str())
    .execute(&mut *tx)
    .await
    .map_err(|e| {
      if let sqlx::Error::Database(ref db_err) = e
        && db_err.code().as_deref() == Some("23505")
      {
        let msg = match db_err.constraint() {
          Some("leaders_email_key") => {
            "a team with this leader email already exists"
          }
          Some("leaders_mobile_cc_mobile_number_key") => {
            "a team with this leader mobile number already exists"
          }
          _ => "unique constraint violation",
        };
        return RepositoryError::Conflict(msg.into());
      }
      RepositoryError::Database(e.to_string())
    })?;

    if let Some(member) = input.member() {
      sqlx::query(
        r#"
                INSERT INTO members (team_id, name, gender, email, mobile_cc, mobile_number,
                                     college_name, degree, department, year_of_study, location)
                VALUES ($1, $2, $3::gender, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
      )
      .bind(team_id)
      .bind(member.name.as_str())
      .bind(match member.gender {
        domain::Gender::Male => "male",
        domain::Gender::Female => "female",
        domain::Gender::Trans => "trans",
        domain::Gender::Others => "others",
      })
      .bind(member.email.as_str())
      .bind(member.mobile_cc.as_str())
      .bind(member.mobile_number.as_str())
      .bind(member.college_name.as_str())
      .bind(member.degree.as_str())
      .bind(member.department.as_str())
      .bind(member.year_of_study.get() as i32)
      .bind(member.location.as_str())
      .execute(&mut *tx)
      .await
      .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e
          && db_err.code().as_deref() == Some("23505")
        {
          let msg = match db_err.constraint() {
            Some("members_email_key") => {
              "a team with this member email already exists"
            }
            Some("members_mobile_cc_mobile_number_key") => {
              "a team with this member mobile number already exists"
            }
            _ => "unique constraint violation",
          };
          return RepositoryError::Conflict(msg.into());
        }
        RepositoryError::Database(e.to_string())
      })?;
    }

    tx.commit()
      .await
      .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(team_id)
  }

  async fn get_team_state(
    &self,
    team_id: Uuid,
  ) -> Result<TeamState, RepositoryError> {
    let state: String =
      sqlx::query_scalar("SELECT state::text FROM teams WHERE id = $1")
        .bind(team_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
          sqlx::Error::RowNotFound => RepositoryError::NotFound("team".into()),
          _ => RepositoryError::Database(e.to_string()),
        })?;
    parse_team_state(&state)
  }

  async fn update_team_state(
    &self,
    team_id: Uuid,
    target: TeamState,
  ) -> Result<bool, RepositoryError> {
    let current = self.get_team_state(team_id).await?;
    if current == target {
      return Ok(false);
    }
    current
      .transition(target)
      .map_err(|e| RepositoryError::Database(e.to_string()))?;
    let current_str = team_state_to_str(current);
    let target_str = team_state_to_str(target);
    let result = sqlx::query(
      "UPDATE teams SET state = $1::team_state WHERE id = $2 AND state::text = $3",
    )
    .bind(target_str)
    .bind(team_id)
    .bind(current_str)
    .execute(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;
    Ok(result.rows_affected() > 0)
  }

  async fn find_by_email(
    &self,
    email: &Email,
  ) -> Result<Option<Uuid>, RepositoryError> {
    let result: Option<(Uuid,)> = sqlx::query_as(
      r#"
            SELECT t.id FROM teams t
            LEFT JOIN leaders l ON l.team_id = t.id
            LEFT JOIN members m ON m.team_id = t.id
            WHERE l.email = $1 OR m.email = $1
            LIMIT 1
            "#,
    )
    .bind(email.as_str())
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(result.map(|r| r.0))
  }

  async fn find_team_id_by_email(
    &self,
    email: &Email,
  ) -> Result<Uuid, RepositoryError> {
    self
      .find_by_email(email)
      .await?
      .ok_or_else(|| RepositoryError::NotFound("team for email".into()))
  }

  async fn set_team_pricing(
    &self,
    team_id: Uuid,
    base_price_paise: i32,
    final_price_paise: i32,
    discount_code: Option<&str>,
  ) -> Result<(), RepositoryError> {
    let r = sqlx::query(
      r#"UPDATE teams SET base_price_paise = $1, final_price_paise = $2, discount_code = $3 WHERE id = $4"#,
    )
    .bind(base_price_paise)
    .bind(final_price_paise)
    .bind(discount_code)
    .bind(team_id)
    .execute(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;
    if r.rows_affected() == 0 {
      return Err(RepositoryError::NotFound("team".into()));
    }
    Ok(())
  }

  async fn find_by_id(
    &self,
    team_id: Uuid,
  ) -> Result<TeamDetails, RepositoryError> {
    let row = sqlx::query(
      r#"SELECT id, registration_type::text as reg_type, team_name, state::text as state,
                      base_price_paise, final_price_paise, discount_code, resend_attempts
               FROM teams WHERE id = $1"#,
    )
    .bind(team_id)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| match e {
      sqlx::Error::RowNotFound => RepositoryError::NotFound("team".into()),
      _ => RepositoryError::Database(e.to_string()),
    })?;

    let reg_type_str: String = row
      .try_get("reg_type")
      .map_err(|e| RepositoryError::Database(e.to_string()))?;
    let team_name: String = row
      .try_get("team_name")
      .map_err(|e| RepositoryError::Database(e.to_string()))?;
    let state_str: String = row
      .try_get("state")
      .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let registration_type = match reg_type_str.as_str() {
      "single" => RegistrationType::Single,
      "couple" => RegistrationType::Couple,
      _ => {
        return Err(RepositoryError::Database(format!(
          "invalid registration_type: {reg_type_str}"
        )));
      }
    };
    let state = parse_team_state(&state_str)?;
    let resend_attempts: i32 = row
      .try_get("resend_attempts")
      .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let leader: LeaderRow =
      sqlx::query_as(
        r#"SELECT id, team_id, name, gender::text AS gender, email, mobile_cc, mobile_number,
                  college_name, degree, department, year_of_study, location
           FROM leaders WHERE team_id = $1"#,
      )
      .bind(team_id)
      .fetch_one(&self.pool)
      .await
      .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let member: Option<MemberRow> =
      sqlx::query_as(
        r#"SELECT id, team_id, name, gender::text AS gender, email, mobile_cc, mobile_number,
                  college_name, degree, department, year_of_study, location
           FROM members WHERE team_id = $1"#,
      )
      .bind(team_id)
      .fetch_optional(&self.pool)
      .await
      .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(TeamDetails {
      id: team_id,
      registration_type,
      team_name,
      state,
      leader_name: leader.name,
      leader_gender: leader.gender,
      leader_email: domain::Email::parse(&leader.email).map_err(|e| {
        RepositoryError::Database(format!("invalid leader email: {e}"))
      })?,
      leader_mobile_cc: leader.mobile_cc,
      leader_mobile_number: leader.mobile_number,
      leader_location: leader.location,
      member_name: member.as_ref().map(|m| m.name.clone()),
      member_gender: member.as_ref().map(|m| m.gender.clone()),
      member_email: member
        .as_ref()
        .map(|m| {
          domain::Email::parse(&m.email).map_err(|e| {
            RepositoryError::Database(format!("invalid member email: {e}"))
          })
        })
        .transpose()?,
      member_mobile_cc: member.as_ref().map(|m| m.mobile_cc.clone()),
      member_mobile_number: member.as_ref().map(|m| m.mobile_number.clone()),
      base_price_paise: row.get("base_price_paise"),
      final_price_paise: row.get("final_price_paise"),
      discount_code: row.get("discount_code"),
      resend_attempts,
    })
  }

  async fn check_team_name_taken(
    &self,
    team_name: &TeamName,
  ) -> Result<bool, RepositoryError> {
    let exists: bool = sqlx::query_scalar(
      r#"SELECT EXISTS(SELECT 1 FROM teams WHERE LOWER(team_name) = LOWER($1))"#,
    )
    .bind(team_name.as_str())
    .fetch_one(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;
    Ok(exists)
  }

  async fn find_by_mobile(
    &self,
    cc: &MobileCc,
    number: &MobileNumber,
  ) -> Result<Option<Uuid>, RepositoryError> {
    let result: Option<(Uuid,)> = sqlx::query_as(
      r#"
            SELECT t.id FROM teams t
            LEFT JOIN leaders l ON l.team_id = t.id
            LEFT JOIN members m ON m.team_id = t.id
            WHERE (l.mobile_cc = $1 AND l.mobile_number = $2)
               OR (m.mobile_cc = $1 AND m.mobile_number = $2)
            LIMIT 1
            "#,
    )
    .bind(cc.as_str())
    .bind(number.as_str())
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;
    Ok(result.map(|r| r.0))
  }

  async fn find_team_by_name(
    &self,
    team_name: &TeamName,
  ) -> Result<Option<TeamLookup>, RepositoryError> {
    let row: Option<(Uuid, String, String)> = sqlx::query_as(
      r#"
            SELECT id, team_name, state::text
            FROM teams
            WHERE LOWER(team_name) = LOWER($1)
            LIMIT 1
            "#,
    )
    .bind(team_name.as_str())
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    row
      .map(|(team_id, name, state_str)| {
        let state = parse_team_state(&state_str)?;
        Ok(TeamLookup {
          team_id,
          team_name: name,
          state,
        })
      })
      .transpose()
  }

  async fn find_team_by_email(
    &self,
    email: &Email,
  ) -> Result<Option<TeamLookup>, RepositoryError> {
    let row: Option<(Uuid, String, String)> = sqlx::query_as(
      r#"
            SELECT t.id, t.team_name, t.state::text
            FROM teams t
            LEFT JOIN leaders l ON l.team_id = t.id
            LEFT JOIN members m ON m.team_id = t.id
            WHERE l.email = $1 OR m.email = $1
            LIMIT 1
            "#,
    )
    .bind(email.as_str())
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    row
      .map(|(team_id, team_name, state_str)| {
        let state = parse_team_state(&state_str)?;
        Ok(TeamLookup {
          team_id,
          team_name,
          state,
        })
      })
      .transpose()
  }

  async fn find_team_by_mobile(
    &self,
    cc: &MobileCc,
    number: &MobileNumber,
  ) -> Result<Option<TeamLookup>, RepositoryError> {
    let row: Option<(Uuid, String, String)> = sqlx::query_as(
      r#"
            SELECT t.id, t.team_name, t.state::text
            FROM teams t
            LEFT JOIN leaders l ON l.team_id = t.id
            LEFT JOIN members m ON m.team_id = t.id
            WHERE (l.mobile_cc = $1 AND l.mobile_number = $2)
               OR (m.mobile_cc = $1 AND m.mobile_number = $2)
            LIMIT 1
            "#,
    )
    .bind(cc.as_str())
    .bind(number.as_str())
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    row
      .map(|(team_id, team_name, state_str)| {
        let state = parse_team_state(&state_str)?;
        Ok(TeamLookup {
          team_id,
          team_name,
          state,
        })
      })
      .transpose()
  }

  async fn is_registration_open(&self) -> Result<bool, RepositoryError> {
    let value: Option<String> = sqlx::query_scalar(
      r#"SELECT value FROM settings WHERE key = 'registration_open'"#,
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;
    Ok(value.as_deref() == Some("true"))
  }

  async fn increment_resend_attempts(
    &self,
    team_id: Uuid,
  ) -> Result<i32, RepositoryError> {
    let attempts: i32 = sqlx::query_scalar(
      r#"UPDATE teams SET resend_attempts = resend_attempts + 1 WHERE id = $1 RETURNING resend_attempts"#,
    )
    .bind(team_id)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(attempts)
  }

  async fn delete_team(&self, team_id: Uuid) -> Result<(), RepositoryError> {
    let r = sqlx::query(r#"DELETE FROM teams WHERE id = $1"#)
      .bind(team_id)
      .execute(&self.pool)
      .await
      .map_err(|e| RepositoryError::Database(e.to_string()))?;

    if r.rows_affected() == 0 {
      return Err(RepositoryError::NotFound("team".into()));
    }

    Ok(())
  }
}

fn parse_team_state(s: &str) -> Result<TeamState, RepositoryError> {
  match s {
    "pending_verification" => Ok(TeamState::PendingVerification),
    "verified" => Ok(TeamState::Verified),
    "payment_pending" => Ok(TeamState::PaymentPending),
    "paid" => Ok(TeamState::Paid),
    "failed" => Ok(TeamState::Failed),
    _ => Err(RepositoryError::Database(format!(
      "invalid team_state: {s}"
    ))),
  }
}

fn team_state_to_str(s: TeamState) -> &'static str {
  match s {
    TeamState::PendingVerification => "pending_verification",
    TeamState::Verified => "verified",
    TeamState::PaymentPending => "payment_pending",
    TeamState::Paid => "paid",
    TeamState::Failed => "failed",
  }
}
