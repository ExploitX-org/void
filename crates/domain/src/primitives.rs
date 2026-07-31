use serde::{Deserialize, Serialize};
use std::fmt;

use uuid::Uuid;

use crate::enums::{Gender, RegistrationType, TeamState};
use crate::error::{ParseError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct TeamName(String);

impl TeamName {
  const DEFAULT_MIN: usize = 3;
  const DEFAULT_MAX: usize = 25;

  pub fn parse(s: impl Into<String>) -> Result<Self> {
    Self::parse_with_len(s, Self::DEFAULT_MIN, Self::DEFAULT_MAX)
  }

  pub fn parse_with_len(
    s: impl Into<String>,
    min: usize,
    max: usize,
  ) -> Result<Self> {
    if min > max {
      return Err(ParseError::Internal(format!(
        "parse_with_len: min ({min}) > max ({max})"
      )));
    }
    let s = s.into().trim().to_string();
    if s.len() < min {
      return Err(ParseError::TooShort("team_name", min));
    }
    if s.len() > max {
      return Err(ParseError::TooLong("team_name", max));
    }
    if s.starts_with('.') || s.ends_with('.') {
      return Err(ParseError::InvalidFormat("team_name"));
    }
    if !s
      .chars()
      .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
      return Err(ParseError::InvalidFormat("team_name"));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for TeamName {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for TeamName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email {
  full: String,
  domain: String,
}

impl Serialize for Email {
  fn serialize<S: serde::Serializer>(
    &self,
    serializer: S,
  ) -> std::result::Result<S::Ok, S::Error> {
    self.full.serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Email {
  fn deserialize<D: serde::Deserializer<'de>>(
    deserializer: D,
  ) -> std::result::Result<Self, D::Error> {
    let s = <String as Deserialize<'de>>::deserialize(deserializer)?;
    Self::parse(s).map_err(|_| serde::de::Error::custom("invalid email format"))
  }
}

impl Email {
  pub fn parse(s: impl Into<String>) -> Result<Self> {
    let full = s.into().trim().to_lowercase();
    if full.is_empty() {
      return Err(ParseError::Empty("email"));
    }
    if !full.contains('@') {
      return Err(ParseError::InvalidFormat("email"));
    }
    if full.len() > 254 {
      return Err(ParseError::TooLong("email", 254));
    }
    let (local, domain) = full
      .split_once('@')
      .ok_or(ParseError::InvalidFormat("email"))?;
    if local.is_empty() || local.len() > 64 {
      return Err(ParseError::InvalidFormat("email"));
    }
    if local.starts_with('.') || local.ends_with('.') || local.contains("..") {
      return Err(ParseError::InvalidFormat("email"));
    }
    if !local
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+'))
    {
      return Err(ParseError::InvalidFormat("email"));
    }
    if domain.is_empty() || !domain.contains('.') {
      return Err(ParseError::InvalidFormat("email"));
    }
    if domain.starts_with('.') || domain.ends_with('.') || domain.contains("..")
    {
      return Err(ParseError::InvalidFormat("email"));
    }
    for part in domain.split('.') {
      if part.is_empty() || part.len() > 63 {
        return Err(ParseError::InvalidFormat("email"));
      }
      if !part.starts_with(|c: char| c.is_ascii_alphanumeric())
        || !part.ends_with(|c: char| c.is_ascii_alphanumeric())
      {
        return Err(ParseError::InvalidFormat("email"));
      }
      if !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(ParseError::InvalidFormat("email"));
      }
    }
    Ok(Self {
      full: full.to_string(),
      domain: domain.to_string(),
    })
  }

  pub fn as_str(&self) -> &str {
    &self.full
  }

  pub fn domain(&self) -> Option<&str> {
    Some(&self.domain)
  }
}

impl TryFrom<String> for Email {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl From<Email> for String {
  fn from(e: Email) -> String {
    e.full
  }
}

impl fmt::Display for Email {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.full.fmt(f)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct MobileCc(String);

impl MobileCc {
  const DEFAULT_MIN: usize = 2;
  const DEFAULT_MAX: usize = 4;

  pub fn parse(s: impl Into<String>) -> Result<Self> {
    Self::parse_with_len(s, Self::DEFAULT_MIN, Self::DEFAULT_MAX)
  }

  pub fn parse_with_len(
    s: impl Into<String>,
    min: usize,
    max: usize,
  ) -> Result<Self> {
    let s = s.into().trim().to_string();
    if s.is_empty() {
      return Err(ParseError::Empty("mobile_cc"));
    }
    if !s.starts_with('+') {
      return Err(ParseError::InvalidFormat("mobile_cc"));
    }
    if !s[1..].chars().all(|c| c.is_ascii_digit()) {
      return Err(ParseError::InvalidFormat("mobile_cc"));
    }
    if s.len() < min {
      return Err(ParseError::TooShort("mobile_cc", min));
    }
    if s.len() > max {
      return Err(ParseError::TooLong("mobile_cc", max));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl Default for MobileCc {
  fn default() -> Self {
    Self("+91".to_string())
  }
}

impl TryFrom<String> for MobileCc {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for MobileCc {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct MobileNumber(String);

impl MobileNumber {
  const DEFAULT_MIN: usize = 10;
  const DEFAULT_MAX: usize = 10;

  pub fn parse(s: impl Into<String>) -> Result<Self> {
    Self::parse_with_len(s, Self::DEFAULT_MIN, Self::DEFAULT_MAX)
  }

  pub fn parse_with_len(
    s: impl Into<String>,
    min: usize,
    max: usize,
  ) -> Result<Self> {
    let s = s.into().trim().to_string();
    if s.is_empty() {
      return Err(ParseError::Empty("mobile_number"));
    }
    if !s.chars().all(|c| c.is_ascii_digit()) {
      return Err(ParseError::InvalidFormat("mobile_number"));
    }
    if s.len() < min {
      return Err(ParseError::TooShort("mobile_number", min));
    }
    if s.len() > max {
      return Err(ParseError::TooLong("mobile_number", max));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for MobileNumber {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for MobileNumber {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct OtpCode(String);

impl std::fmt::Debug for OtpCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("OtpCode").field(&"***").finish()
  }
}

impl OtpCode {
  const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  const LENGTH: usize = 8;

  pub fn generate() -> Result<Self> {
    let charset_len = Self::CHARSET.len();
    let max_valid = 256 - (256 % charset_len);
    let mut buf = [0u8; Self::LENGTH];
    loop {
      getrandom::getrandom(&mut buf)
        .map_err(|e| ParseError::Internal(e.to_string()))?;
      if buf.iter().all(|&b| (b as usize) < max_valid) {
        break;
      }
    }
    let code: String = buf
      .iter()
      .map(|&b| Self::CHARSET[(b as usize) % charset_len] as char)
      .collect();
    Ok(Self(code))
  }

  pub fn parse(s: impl Into<String>) -> Result<Self> {
    let s = s.into().trim().to_uppercase();
    if s.len() != Self::LENGTH {
      return Err(ParseError::InvalidLength(
        "otp_code",
        Self::LENGTH,
        Self::LENGTH,
      ));
    }
    if !s.chars().all(|c| c.is_ascii_alphanumeric()) {
      return Err(ParseError::InvalidFormat("otp_code"));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for OtpCode {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for OtpCode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "***")
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Name(String);

impl Name {
  pub fn parse(s: impl Into<String>) -> Result<Self> {
    let s = s.into().trim().to_string();
    if s.is_empty() {
      return Err(ParseError::Empty("name"));
    }
    if s.len() > 100 {
      return Err(ParseError::TooLong("name", 100));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for Name {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for Name {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct CollegeName(String);

impl CollegeName {
  pub fn parse(s: impl Into<String>) -> Result<Self> {
    let s = s.into().trim().to_string();
    if s.is_empty() {
      return Err(ParseError::Empty("college_name"));
    }
    if s.len() > 200 {
      return Err(ParseError::TooLong("college_name", 200));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for CollegeName {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for CollegeName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Degree(String);

impl Degree {
  pub fn parse(s: impl Into<String>) -> Result<Self> {
    let s = s.into().trim().to_string();
    if s.is_empty() {
      return Err(ParseError::Empty("degree"));
    }
    if s.len() > 100 {
      return Err(ParseError::TooLong("degree", 100));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for Degree {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for Degree {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Department(String);

impl Department {
  pub fn parse(s: impl Into<String>) -> Result<Self> {
    let s = s.into().trim().to_string();
    if s.is_empty() {
      return Err(ParseError::Empty("department"));
    }
    if s.len() > 100 {
      return Err(ParseError::TooLong("department", 100));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for Department {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for Department {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "i32")]
pub struct YearOfStudy(u8);

impl YearOfStudy {
  pub fn parse(n: u8) -> Result<Self> {
    if !(1..=7).contains(&n) {
      return Err(ParseError::OutOfRange("year_of_study", 1, 7));
    }
    Ok(Self(n))
  }

  pub fn get(&self) -> u8 {
    self.0
  }
}

impl TryFrom<i32> for YearOfStudy {
  type Error = ParseError;
  fn try_from(n: i32) -> Result<Self> {
    let n_u8: u8 = n
      .try_into()
      .map_err(|_| ParseError::OutOfRange("year_of_study", 1, 7))?;
    Self::parse(n_u8)
  }
}

impl fmt::Display for YearOfStudy {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Location(String);

impl Location {
  pub fn parse(s: impl Into<String>) -> Result<Self> {
    let s = s.into().trim().to_string();
    if s.is_empty() {
      return Err(ParseError::Empty("location"));
    }
    if s.len() > 200 {
      return Err(ParseError::TooLong("location", 200));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for Location {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for Location {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "i32")]
pub struct Amount(i32);

impl Amount {
  pub fn parse(paise: i32) -> Result<Self> {
    if paise < 0 {
      return Err(ParseError::OutOfRange("amount", 0, i32::MAX));
    }
    Ok(Self(paise))
  }

  pub fn get(&self) -> i32 {
    self.0
  }
}

impl TryFrom<i32> for Amount {
  type Error = ParseError;
  fn try_from(paise: i32) -> Result<Self> {
    Self::parse(paise)
  }
}

impl fmt::Display for Amount {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct InvoiceNo(String);

impl InvoiceNo {
  pub fn parse(s: impl Into<String>) -> Result<Self> {
    let s = s.into().trim().to_string();
    if s.is_empty() {
      return Err(ParseError::Empty("invoice_no"));
    }
    if s.len() > 50 {
      return Err(ParseError::TooLong("invoice_no", 50));
    }
    Ok(Self(s))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for InvoiceNo {
  type Error = ParseError;
  fn try_from(s: String) -> Result<Self> {
    Self::parse(s)
  }
}

impl fmt::Display for InvoiceNo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonData {
  pub name: Name,
  pub gender: Gender,
  pub email: Email,
  pub mobile_cc: MobileCc,
  pub mobile_number: MobileNumber,
  pub college_name: CollegeName,
  pub degree: Degree,
  pub department: Department,
  pub year_of_study: YearOfStudy,
  pub location: Location,
}

#[derive(Debug, Clone)]
pub enum RegistrationInput {
  Single {
    team_name: TeamName,
    leader: PersonData,
  },
  Couple {
    team_name: TeamName,
    leader: PersonData,
    member: Box<PersonData>,
  },
}

impl RegistrationInput {
  pub fn team_name(&self) -> &TeamName {
    match self {
      Self::Single { team_name, .. } | Self::Couple { team_name, .. } => {
        team_name
      }
    }
  }

  pub fn leader(&self) -> &PersonData {
    match self {
      Self::Single { leader, .. } | Self::Couple { leader, .. } => leader,
    }
  }

  pub fn member(&self) -> Option<&PersonData> {
    match self {
      Self::Single { .. } => None,
      Self::Couple { member, .. } => Some(member.as_ref()),
    }
  }

  pub fn registration_type(&self) -> RegistrationType {
    match self {
      Self::Single { .. } => RegistrationType::Single,
      Self::Couple { .. } => RegistrationType::Couple,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscountInfo {
  pub base_price_paise: i32,
  pub final_price_paise: i32,
  pub discount_amount: i32,
  pub coupon_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TeamLookup {
  pub team_id: Uuid,
  pub team_name: String,
  pub state: TeamState,
}
