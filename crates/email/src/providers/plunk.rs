use std::sync::OnceLock;
use std::time::Duration;

use crate::content;
use crate::{EmailError, EmailService};
use domain::{Email, OtpCode};

#[derive(Clone)]
pub struct PlunkEmailClient {
  http: reqwest::Client,
  secret_key: String,
  from_email: String,
  from_name: String,
}

#[derive(Clone, serde::Serialize)]
struct PlunkAttachment {
  filename: String,
  content: String,
  #[serde(rename = "contentType")]
  content_type: String,
  #[serde(skip_serializing_if = "Option::is_none", rename = "contentId")]
  content_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  disposition: Option<String>,
}

#[derive(serde::Serialize)]
struct PlunkPayload {
  to: String,
  subject: String,
  body: String,
  from: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  name: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  attachments: Vec<PlunkAttachment>,
}

#[derive(serde::Deserialize)]
struct PlunkResponse {
  success: Option<bool>,
}

impl PlunkEmailClient {
  pub fn new(
    secret_key: String,
    from_email: String,
    from_name: String,
  ) -> Result<Self, EmailError> {
    let from_name = from_name.replace(['"', '\\'], "");
    Ok(Self {
      http: reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| EmailError::Transport(e.to_string()))?,
      secret_key,
      from_email,
      from_name,
    })
  }

  fn inline_image(
    name: &str,
    data: &[u8],
    content_id: &str,
  ) -> PlunkAttachment {
    PlunkAttachment {
      filename: name.to_string(),
      content: base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        data,
      ),
      content_type: "image/png".into(),
      content_id: Some(content_id.to_string()),
      disposition: Some("inline".into()),
    }
  }

  fn base_attachments() -> &'static Vec<PlunkAttachment> {
    static ATTACHMENTS: OnceLock<Vec<PlunkAttachment>> = OnceLock::new();
    ATTACHMENTS.get_or_init(|| {
      vec![
        Self::inline_image("logo.png", content::LOGO_PNG, "logo@exploitx"),
        Self::inline_image(
          "mascot.png",
          content::MASCOT_PNG,
          "mascot@exploitx",
        ),
      ]
    })
  }

  async fn send_payload(
    &self,
    payload: &PlunkPayload,
  ) -> Result<(), EmailError> {
    let resp = self
      .http
      .post("https://next-api.useplunk.com/v1/send")
      .header("Authorization", format!("Bearer {}", self.secret_key))
      .json(payload)
      .send()
      .await
      .map_err(|e| EmailError::Transport(format!("request failed: {e}")))?;

    let status = resp.status();
    let body_text = resp.text().await.map_err(|e| {
      EmailError::Transport(format!("read response body failed: {e}"))
    })?;
    tracing::debug!("Plunk API response (HTTP {status}): {body_text}");

    let body: PlunkResponse =
      serde_json::from_str(&body_text).map_err(|e| {
        EmailError::Api(format!("parse response (HTTP {status}) failed: {e}"))
      })?;

    if !body.success.unwrap_or(false) {
      return Err(EmailError::Api(format!(
        "Plunk API error (HTTP {status}): {body_text}"
      )));
    }

    Ok(())
  }
}

impl EmailService for PlunkEmailClient {
  async fn send_otp(
    &self,
    to: &Email,
    otp: &OtpCode,
    ttl_secs: i64,
  ) -> Result<(), EmailError> {
    let html = content::render_otp_email(otp.as_str(), ttl_secs);

    let payload = PlunkPayload {
      to: to.as_str().to_string(),
      subject: "Your Verification Code for Into The Void 2.0".into(),
      body: html,
      from: self.from_email.clone(),
      // plunk api requires the sender name to be pre-quoted
      name: Some(format!("\"{}\"", self.from_name)),
      attachments: Self::base_attachments().clone(),
    };

    self.send_payload(&payload).await
  }

  async fn send_invoice(
    &self,
    to: &Email,
    leader_name: &str,
    invoice_no: &str,
    team_name: &str,
    reg_type: &str,
    amount: i32,
    pdf_data: Option<Vec<u8>>,
  ) -> Result<(), EmailError> {
    let has_pdf = pdf_data.is_some();
    let html = content::render_invoice_email(
      leader_name,
      invoice_no,
      has_pdf,
      team_name,
      reg_type,
      amount,
    );

    let mut attachments = Self::base_attachments().clone();

    if let Some(pdf) = pdf_data {
      attachments.push(PlunkAttachment {
        filename: format!("invoice_{invoice_no}.pdf"),
        content: base64::Engine::encode(
          &base64::engine::general_purpose::STANDARD,
          &pdf,
        ),
        content_type: "application/pdf".into(),
        content_id: None,
        disposition: Some("attachment".into()),
      });
    }

    let payload = PlunkPayload {
      to: to.as_str().to_string(),
      subject: "Registration Confirmed – Into The Void 2.0".into(),
      body: html,
      from: self.from_email.clone(),
      // plunk api requires the sender name to be pre-quoted
      name: Some(format!("\"{}\"", self.from_name)),
      attachments,
    };

    self.send_payload(&payload).await?;
    tracing::debug!(
      "plunk invoice email sent{}",
      if has_pdf { " with PDF" } else { "" }
    );
    Ok(())
  }
}
