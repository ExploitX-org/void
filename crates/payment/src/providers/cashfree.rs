use std::time::Duration;

use domain::Amount;
use serde::Deserialize;
use serde::Deserializer;

fn deserialize_int_as_string<'de, D>(
  deserializer: D,
) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  #[serde(untagged)]
  enum IntOrString {
    Int(i64),
    String(String),
  }
  match IntOrString::deserialize(deserializer)? {
    IntOrString::Int(i) => Ok(i.to_string()),
    IntOrString::String(s) => Ok(s),
  }
}

use crate::{
  PaymentCreateResponse, PaymentCustomerInfo, PaymentError, PaymentGateway,
  PaymentStatusResponse,
};

#[derive(Clone)]
pub struct CashfreeClient {
  http: reqwest::Client,
  api_id: String,
  api_secret: String,
  base_url: String,
  is_production: bool,
}

impl std::fmt::Debug for CashfreeClient {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CashfreeClient")
      .field("api_id", &"***")
      .field("api_secret", &"***")
      .field("base_url", &self.base_url)
      .field("is_production", &self.is_production)
      .finish_non_exhaustive()
  }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CfOrderResponse {
  #[serde(deserialize_with = "deserialize_int_as_string")]
  cf_order_id: String,
  order_amount: f64,
  order_status: String,
  payment_session_id: Option<String>,
  order_token: Option<String>,
  payment_link: Option<String>,
  order_id: String,
  order_currency: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CfOrderStatusResponse {
  order_id: String,
  order_amount: f64,
  order_status: String,
  order_currency: String,
  #[serde(deserialize_with = "deserialize_int_as_string")]
  cf_order_id: String,
}

impl CashfreeClient {
  pub fn new(
    api_id: String,
    api_secret: String,
    is_production: bool,
  ) -> Result<Self, PaymentError> {
    let base_url = if is_production {
      "https://api.cashfree.com/pg"
    } else {
      "https://sandbox.cashfree.com/pg"
    };

    Ok(Self {
      http: reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| PaymentError::Network(e.to_string()))?,
      api_id,
      api_secret,
      base_url: base_url.into(),
      is_production,
    })
  }

  fn auth_headers(&self) -> Result<reqwest::header::HeaderMap, PaymentError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
      "x-client-id",
      reqwest::header::HeaderValue::from_str(&self.api_id).map_err(|e| {
        PaymentError::Gateway(format!("invalid Cashfree API ID in header: {e}"))
      })?,
    );
    headers.insert(
      "x-client-secret",
      reqwest::header::HeaderValue::from_str(&self.api_secret).map_err(
        |e| {
          PaymentError::Gateway(format!(
            "invalid Cashfree API secret in header: {e}"
          ))
        },
      )?,
    );
    headers.insert(
      "x-api-version",
      reqwest::header::HeaderValue::from_static("2022-01-01"),
    );
    headers.insert(
      reqwest::header::ACCEPT,
      reqwest::header::HeaderValue::from_static("application/json"),
    );
    Ok(headers)
  }
}

impl CashfreeClient {
  fn payment_page_url(&self, session_id: &str) -> String {
    let base = if self.is_production {
      "https://pay.cashfree.com/pg"
    } else {
      "https://sandbox.cashfree.com/pg"
    };
    format!("{base}/view/sessions/checkout/web/{session_id}")
  }
}

impl PaymentGateway for CashfreeClient {
  async fn create(
    &self,
    order_id: &str,
    amount: Amount,
    customer: PaymentCustomerInfo,
    callback_url: &str,
  ) -> Result<PaymentCreateResponse, PaymentError> {
    let body = serde_json::json!({
        "order_id": order_id,
        "order_amount": format!("{}.{:02}", amount.get() / 100, amount.get() % 100),
        "order_currency": "INR",
        "customer_details": {
            "customer_id": order_id,
            "customer_name": customer.name,
            "customer_email": customer.email,
            "customer_phone": customer.phone,
        },
        "order_meta": {
            "return_url": callback_url,
            "notify_url": callback_url,
        },
    });

    let resp = self
      .http
      .post(format!("{}/orders", self.base_url))
      .headers(self.auth_headers()?)
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
        "cashfree create order failed (HTTP {status})"
      )));
    }

    let cf_resp: CfOrderResponse = serde_json::from_str(&text)
      .map_err(|e| PaymentError::Gateway(e.to_string()))?;

    let payment_link = match cf_resp.payment_link {
      Some(link) => link,
      None => {
        let sid = cf_resp.payment_session_id.ok_or_else(|| {
          PaymentError::Gateway(
            "Cashfree response missing payment_session_id".into(),
          )
        })?;
        self.payment_page_url(&sid)
      }
    };

    Ok(PaymentCreateResponse {
      order_id: order_id.to_string(),
      payment_link,
      provider: "cashfree".into(),
    })
  }

  async fn status(
    &self,
    order_id: &str,
  ) -> Result<PaymentStatusResponse, PaymentError> {
    let resp = self
      .http
      .get(format!("{}/orders/{}", self.base_url, order_id))
      .headers(self.auth_headers()?)
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
        "cashfree status failed (HTTP {status})"
      )));
    }

    let cf_resp: CfOrderStatusResponse = serde_json::from_str(&text)
      .map_err(|e| PaymentError::Gateway(e.to_string()))?;

    let amount = (cf_resp.order_amount * 100.0 + 0.5).floor() as i64;
    Ok(PaymentStatusResponse {
      order_id: cf_resp.order_id,
      status: cf_resp.order_status,
      amount,
      paid_amount: None,
    })
  }

  async fn verify(
    &self,
    _payment_id: &str,
    _signature: &str,
  ) -> Result<bool, PaymentError> {
    Err(PaymentError::Gateway(
      "Cashfree verify not implemented".into(),
    ))
  }
}
