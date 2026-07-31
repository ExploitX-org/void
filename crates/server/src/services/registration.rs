use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, Utc};
use domain::{
  OtpCode, RegistrationInput, RegistrationType, TeamLookup, TeamRepository,
  TeamState, VerificationRepository, VerificationTarget,
};
use payment::PricingService;
use uuid::Uuid;

use crate::AppState;
use crate::error::ApiError;
use crate::services::verification::VerificationService;
use crate::types::ParsedResendRequest;

pub enum RegisterAction {
  Created { team_id: String },
  Resent { team_id: String },
}

struct FieldCheck {
  label: &'static str,
  lookup: Option<TeamLookup>,
}

impl FieldCheck {
  fn team_id(&self) -> Option<Uuid> {
    self.lookup.as_ref().map(|t| t.team_id)
  }
}

struct PartitionBlock {
  team: TeamLookup,
  fields: Vec<&'static str>,
}

struct Partition {
  free_fields: Vec<&'static str>,
  blocks: Vec<PartitionBlock>,
}

pub struct RegistrationService;

impl RegistrationService {
  pub async fn register(
    state: &AppState,
    input: RegistrationInput,
    portal_start: DateTime<FixedOffset>,
    base_price_paise: i32,
    otp_ttl_secs: i64,
  ) -> Result<RegisterAction, ApiError> {
    let is_couple =
      matches!(input.registration_type(), RegistrationType::Couple);
    let contact_labels: &[&str] = if is_couple {
      &[
        "Leader email",
        "Leader mobile",
        "Member email",
        "Member mobile",
      ]
    } else {
      &["Leader email", "Leader mobile"]
    };

    let checks = build_field_checks(&input, &state.team_repo).await?;
    let partition = build_partition(&checks);

    match evaluate_partition(
      &partition,
      contact_labels,
      input.team_name().as_str(),
    ) {
      PartitionOutcome::Create => {
        let team_id = Self::create_with_pricing_and_otp(
          state,
          &input,
          portal_start,
          base_price_paise,
          otp_ttl_secs,
        )
        .await?;
        Ok(RegisterAction::Created { team_id })
      }

      PartitionOutcome::Resend(team_id) => {
        VerificationService::resend_otp(state, ParsedResendRequest { team_id })
          .await?;
        Ok(RegisterAction::Resent {
          team_id: team_id.to_string(),
        })
      }

      PartitionOutcome::Conflict(message) => Err(ApiError::Conflict(message)),
    }
  }

  async fn create_with_pricing_and_otp(
    state: &AppState,
    input: &RegistrationInput,
    portal_start: DateTime<FixedOffset>,
    base_price_paise: i32,
    otp_ttl_secs: i64,
  ) -> Result<String, ApiError> {
    let pricing = PricingService::new(portal_start, base_price_paise);
    let (_, discount) = pricing.calculate_amount(
      input.registration_type(),
      &input.leader().email,
      input.member().map(|m| &m.email),
    )?;

    let team_id = state.team_repo.create_team(input).await?;

    let result = async {
      state
        .team_repo
        .set_team_pricing(
          team_id,
          discount.base_price_paise,
          discount.final_price_paise,
          discount.coupon_code.as_deref(),
        )
        .await?;

      let expires_at = Utc::now() + chrono::Duration::seconds(otp_ttl_secs);

      let leader_otp = OtpCode::generate()?;
      let leader_hash =
        crate::crypto::sha256_hex(leader_otp.as_str().as_bytes());
      let team_id_str = team_id.to_string();

      state
        .verification_repo
        .store_otp(
          team_id,
          VerificationTarget::Leader,
          &input.leader().email,
          &leader_hash,
          expires_at,
        )
        .await?;

      state
        .email_client
        .send_otp(&input.leader().email, &leader_otp, otp_ttl_secs)
        .await
        .map_err(|e| {
          ApiError::Internal(format!("failed to send otp email: {e}"))
        })?;

      if let Some(member) = input.member() {
        let member_otp = OtpCode::generate()?;
        let member_hash =
          crate::crypto::sha256_hex(member_otp.as_str().as_bytes());
        state
          .verification_repo
          .store_otp(
            team_id,
            VerificationTarget::Member,
            &member.email,
            &member_hash,
            expires_at,
          )
          .await?;

        state
          .email_client
          .send_otp(&member.email, &member_otp, otp_ttl_secs)
          .await
          .map_err(|e| {
            ApiError::Internal(format!(
              "failed to send otp email to member: {e}"
            ))
          })?;
      }

      Ok::<String, ApiError>(team_id_str)
    }
    .await;

    match result {
      Ok(id) => Ok(id),
      Err(e) => {
        let _ = state.team_repo.delete_team(team_id).await;
        Err(e)
      }
    }
  }
}

enum PartitionOutcome {
  Create,
  Resend(Uuid),
  Conflict(String),
}

async fn build_field_checks(
  input: &RegistrationInput,
  team_repo: &impl TeamRepository,
) -> Result<Vec<FieldCheck>, ApiError> {
  let leader = input.leader();
  let mut checks = vec![
    FieldCheck {
      label: "Team name",
      lookup: team_repo.find_team_by_name(input.team_name()).await?,
    },
    FieldCheck {
      label: "Leader email",
      lookup: team_repo.find_team_by_email(&leader.email).await?,
    },
    FieldCheck {
      label: "Leader mobile",
      lookup: team_repo
        .find_team_by_mobile(&leader.mobile_cc, &leader.mobile_number)
        .await?,
    },
  ];

  if let Some(member) = input.member() {
    checks.push(FieldCheck {
      label: "Member email",
      lookup: team_repo.find_team_by_email(&member.email).await?,
    });
    checks.push(FieldCheck {
      label: "Member mobile",
      lookup: team_repo
        .find_team_by_mobile(&member.mobile_cc, &member.mobile_number)
        .await?,
    });
  }

  Ok(checks)
}

fn build_partition(checks: &[FieldCheck]) -> Partition {
  let mut block_map: HashMap<
    Option<Uuid>,
    (Option<TeamLookup>, Vec<&'static str>),
  > = HashMap::new();

  for check in checks {
    let key = check.team_id();
    let entry = block_map.entry(key).or_insert((None, vec![]));
    if entry.0.is_none() {
      entry.0 = check.lookup.clone();
    }
    entry.1.push(check.label);
  }

  let mut free_fields: Vec<&'static str> = vec![];
  let mut blocks: Vec<PartitionBlock> = vec![];

  for (key, (team_opt, fields)) in block_map {
    if key.is_none() {
      free_fields = fields;
    } else if let Some(team) = team_opt {
      blocks.push(PartitionBlock { team, fields });
    }
  }

  Partition {
    free_fields,
    blocks,
  }
}

fn evaluate_partition(
  partition: &Partition,
  contact_labels: &[&str],
  input_team_name: &str,
) -> PartitionOutcome {
  if partition.blocks.is_empty() {
    return PartitionOutcome::Create;
  }

  if let Some(contact_team_id) = find_contacts_team(partition, contact_labels) {
    let Some(block) = partition
      .blocks
      .iter()
      .find(|b| b.team.team_id == contact_team_id)
    else {
      return PartitionOutcome::Conflict("internal error".into());
    };

    let name_matches =
      block.team.team_name.eq_ignore_ascii_case(input_team_name);
    let state = block.team.state;

    if name_matches && state == TeamState::PendingVerification {
      return PartitionOutcome::Resend(contact_team_id);
    }
  }

  if partition.blocks.len() == 1 && partition.free_fields.is_empty() {
    let Some(block) = partition.blocks.first() else {
      return PartitionOutcome::Conflict("internal error".into());
    };
    let name_matches =
      block.team.team_name.eq_ignore_ascii_case(input_team_name);

    if name_matches {
      return PartitionOutcome::Conflict(state_message(block.team.state));
    }
  }

  PartitionOutcome::Conflict(build_message(partition, input_team_name))
}

fn find_contacts_team(
  partition: &Partition,
  contact_labels: &[&str],
) -> Option<Uuid> {
  let mut found_team_id: Option<Uuid> = None;

  for label in contact_labels {
    if partition.free_fields.contains(label) {
      return None;
    }

    let b = partition.blocks.iter().find(|b| b.fields.contains(label))?;

    match found_team_id {
      None => found_team_id = Some(b.team.team_id),
      Some(prev) if prev != b.team.team_id => return None,
      _ => {}
    }
  }

  found_team_id
}

fn state_message(state: TeamState) -> String {
  match state {
    TeamState::Verified => {
      "This team has already been verified. Please proceed to payment."
        .into()
    }
    TeamState::PaymentPending => {
      "Payment is already being processed for this team.".into()
    }
    TeamState::Paid => {
      "This team has already completed registration and payment.".into()
    }
    TeamState::Failed => {
      "Your previous registration attempt failed. Please register with new credentials."
        .into()
    }
    TeamState::PendingVerification => {
      "This team is pending verification. Please check your email for the OTP."
        .into()
    }
  }
}

fn build_message(partition: &Partition, input_team_name: &str) -> String {
  let mut parts: Vec<String> = vec![];

  for block in &partition.blocks {
    let field_desc = format_fields(&block.fields);
    let name_matches =
      block.team.team_name.eq_ignore_ascii_case(input_team_name);

    let sentence = if name_matches {
      format!(
        "{} {} already registered with this team.",
        field_desc,
        verb_phrase(&block.fields),
      )
    } else {
      format!(
        "{} {} registered with a different team ('{}').",
        field_desc,
        verb_phrase(&block.fields),
        block.team.team_name,
      )
    };
    parts.push(sentence);
  }

  if !partition.free_fields.is_empty() {
    let free_desc = format_fields(&partition.free_fields);
    parts.push(format!(
      "{} {} available.",
      free_desc,
      verb_phrase(&partition.free_fields),
    ));
  }

  parts.join(" ")
}

fn format_fields(fields: &[&str]) -> String {
  match fields.len() {
    0 => String::new(),
    1 => fields[0].to_string(),
    2 => format!("{} and {}", fields[0], fields[1]),
    n => {
      let mut s = String::from(fields[0]);
      for f in &fields[1..n - 1] {
        s.push_str(", ");
        s.push_str(f);
      }
      s.push_str(", and ");
      s.push_str(fields[n - 1]);
      s
    }
  }
}

fn verb_phrase(fields: &[&str]) -> &'static str {
  if fields.len() == 1 { "is" } else { "are" }
}
