use std::time::Duration;

use base64::Engine;
use domain::Amount;
use serde::Deserialize;

use crate::{
  PaymentCreateResponse, PaymentCustomerInfo, PaymentError, PaymentGateway,
  PaymentStatusResponse,
};

#[derive(Clone)]
pub struct RazorpayClient {
  http: reqwest::Client,
  api_key: String,
  api_secret: String,
}

impl std::fmt::Debug for RazorpayClient {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RazorpayClient")
      .field("api_key", &"***")
      .field("api_secret", &"***")
      .finish_non_exhaustive()
  }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RzpOrderResponse {
  id: String,
  amount: i32,
  amount_due: i32,
  amount_paid: i32,
  status: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RzpPaymentLinkResponse {
  id: String,
  short_url: String,
  order_id: String,
}

impl RazorpayClient {
  pub fn new(
    api_key: String,
    api_secret: String,
  ) -> Result<Self, PaymentError> {
    Ok(Self {
      http: reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| PaymentError::Network(e.to_string()))?,
      api_key,
      api_secret,
    })
  }

  fn basic_auth_header(&self) -> String {
    let credentials = format!("{}:{}", self.api_key, self.api_secret);
    format!(
      "Basic {}",
      base64::engine::general_purpose::STANDARD.encode(credentials)
    )
  }
}

impl PaymentGateway for RazorpayClient {
  async fn create(
    &self,
    order_id: &str,
    amount: Amount,
    customer: PaymentCustomerInfo,
    callback_url: &str,
  ) -> Result<PaymentCreateResponse, PaymentError> {
    let body = serde_json::json!({
        "amount": amount.get(),
        "currency": "INR",
        "accept_partial": false,
        "reference_id": order_id,
        "description": format!("Team registration for {}", customer.team_name),
        "customer": {
            "name": customer.name,
            "email": customer.email,
            "contact": customer.phone,
        },
        "notify": { "sms": false, "email": true },
        "reminder_enable": false,
        "callback_url": callback_url,
        "callback_method": "get",
    });

    let resp = self
      .http
      .post("https://api.razorpay.com/v1/payment_links/")
      .header("Authorization", self.basic_auth_header())
      .json(&body)
      .send()
      .await
      .map_err(|e| PaymentError::Network(e.to_string()))?;

    let status = resp.status();
    let text = resp
      .text()
      .await
      .map_err(|e| PaymentError::Network(e.to_string()))?;

    if !status.is_success() {
      return Err(PaymentError::Gateway(format!(
        "razorpay payment link creation failed (HTTP {status})"
      )));
    }

    let pl_data: RzpPaymentLinkResponse = serde_json::from_str(&text)
      .map_err(|e| PaymentError::Gateway(e.to_string()))?;

    Ok(PaymentCreateResponse {
      order_id: pl_data.order_id,
      payment_link: pl_data.short_url,
      provider: "razorpay".into(),
    })
  }

  async fn status(
    &self,
    order_id: &str,
  ) -> Result<PaymentStatusResponse, PaymentError> {
    let resp = self
      .http
      .get(format!("https://api.razorpay.com/v1/orders/{}", order_id))
      .header("Authorization", self.basic_auth_header())
      .send()
      .await
      .map_err(|e| PaymentError::Network(e.to_string()))?;

    let status = resp.status();
    let text = resp
      .text()
      .await
      .map_err(|e| PaymentError::Network(e.to_string()))?;

    if !status.is_success() {
      return Err(PaymentError::Gateway(format!(
        "razorpay status failed (HTTP {status})"
      )));
    }

    let rzp_resp: RzpOrderResponse = serde_json::from_str(&text)
      .map_err(|e| PaymentError::Gateway(e.to_string()))?;

    Ok(PaymentStatusResponse {
      order_id: order_id.to_string(),
      status: rzp_resp.status,
      amount: rzp_resp.amount as i64,
      paid_amount: if rzp_resp.amount_paid > 0 {
        Some(rzp_resp.amount_paid as i64)
      } else {
        None
      },
    })
  }

  async fn verify(
    &self,
    _payment_id: &str,
    _signature: &str,
  ) -> Result<bool, PaymentError> {
    Err(PaymentError::Gateway(
      "Razorpay verify requires order_id which is not provided by the current trait".into(),
    ))
  }
}
