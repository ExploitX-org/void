use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationType {
  Single,
  Couple,
}

impl RegistrationType {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Single => "single",
      Self::Couple => "couple",
    }
  }
}

impl TryFrom<String> for RegistrationType {
  type Error = crate::error::ParseError;
  fn try_from(s: String) -> Result<Self, Self::Error> {
    match s.to_lowercase().as_str() {
      "single" => Ok(Self::Single),
      "couple" => Ok(Self::Couple),
      _ => Err(crate::error::ParseError::InvalidFormat("registration_type")),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
  #[serde(rename = "male")]
  Male,
  #[serde(rename = "female")]
  Female,
  #[serde(rename = "trans")]
  Trans,
  #[serde(rename = "others")]
  Others,
}

impl Gender {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Male => "male",
      Self::Female => "female",
      Self::Trans => "trans",
      Self::Others => "others",
    }
  }
}

impl TryFrom<String> for Gender {
  type Error = crate::error::ParseError;
  fn try_from(s: String) -> Result<Self, Self::Error> {
    match s.to_lowercase().as_str() {
      "male" => Ok(Self::Male),
      "female" => Ok(Self::Female),
      "trans" => Ok(Self::Trans),
      "others" => Ok(Self::Others),
      _ => Err(crate::error::ParseError::InvalidFormat("gender")),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentProvider {
  Cashfree,
  Razorpay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
  Initiated,
  Completed,
  Failed,
  Refunded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationTarget {
  Leader,
  Member,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamState {
  PendingVerification,
  Verified,
  PaymentPending,
  Paid,
  Failed,
}
