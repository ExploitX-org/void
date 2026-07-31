// Traits use async fn; all trait objects are Send+Sync so the lint is overly cautious.
#![allow(async_fn_in_trait)]

pub mod pricing;
pub mod providers;

pub use pricing::PricingService;
pub use providers::*;

pub fn generate_order_id() -> String {
  let short = &Uuid::new_v4().to_string()[..12];
  format!("VOID-{short}")
}

use domain::Amount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PaymentCustomerInfo {
  pub name: String,
  pub email: String,
  pub phone: String,
  pub team_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
  #[error("payment gateway error: {0}")]
  Gateway(String),

  #[error("invalid payment signature")]
  InvalidSignature,

  #[error("network error: {0}")]
  Network(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCreateResponse {
  pub order_id: String,
  pub payment_link: String,
  pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentStatusResponse {
  pub order_id: String,
  pub status: String,
  pub amount: i64,
  pub paid_amount: Option<i64>,
}

pub trait PaymentGateway: Send + Sync {
  async fn create(
    &self,
    order_id: &str,
    amount: Amount,
    customer: PaymentCustomerInfo,
    callback_url: &str,
  ) -> Result<PaymentCreateResponse, PaymentError>;

  async fn status(
    &self,
    order_id: &str,
  ) -> Result<PaymentStatusResponse, PaymentError>;

  async fn verify(
    &self,
    payment_id: &str,
    signature: &str,
  ) -> Result<bool, PaymentError>;
}
