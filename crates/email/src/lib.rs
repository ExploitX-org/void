// Traits use async fn; all trait objects are Send+Sync so the lint is overly cautious.
#![allow(async_fn_in_trait)]

pub mod content;
pub mod providers;

pub use content::*;
pub use providers::*;

use domain::{Email, OtpCode};

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
  #[error("smtp transport error: {0}")]
  Transport(String),

  #[error("api error: {0}")]
  Api(String),

  #[error("template error: {0}")]
  Template(String),
}

pub trait EmailService: Send + Sync {
  async fn send_otp(
    &self,
    to: &Email,
    otp: &OtpCode,
    ttl_secs: i64,
  ) -> Result<(), EmailError>;

  #[allow(clippy::too_many_arguments)]
  async fn send_invoice(
    &self,
    to: &Email,
    leader_name: &str,
    invoice_no: &str,
    team_name: &str,
    reg_type: &str,
    amount: i32,
    pdf_data: Option<Vec<u8>>,
  ) -> Result<(), EmailError>;
}
