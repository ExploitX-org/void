use lettre::address::Address;
use lettre::message::header::{ContentDisposition, ContentId, ContentType};
use lettre::message::{Body, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::content;
use crate::{EmailError, EmailService};
use domain::{Email, OtpCode};

#[derive(Clone)]
pub struct GoogleEmailClient {
  mailer: AsyncSmtpTransport<Tokio1Executor>,
  from_name: String,
  from_email: String,
}

impl GoogleEmailClient {
  pub fn new(
    smtp_host: String,
    smtp_port: u16,
    smtp_secure: bool,
    smtp_user: String,
    smtp_pass: String,
    from_name: String,
    from_email: String,
  ) -> Result<Self, EmailError> {
    let creds = Credentials::new(smtp_user, smtp_pass);
    let builder = if smtp_secure {
      AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
        .map_err(|e| EmailError::Transport(e.to_string()))?
    } else {
      AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
        .map_err(|e| EmailError::Transport(e.to_string()))?
    };
    let mailer = builder.credentials(creds).port(smtp_port).build();
    Ok(Self {
      mailer,
      from_name,
      from_email,
    })
  }

  fn html_part(html: String) -> SinglePart {
    SinglePart::builder()
      .header(ContentType::TEXT_HTML)
      .body(html)
  }

  fn logo_part() -> SinglePart {
    SinglePart::builder()
      .header(
        ContentType::parse("image/png; name=\"logo.png\"")
          .expect("invalid MIME type constant"),
      )
      .header(ContentId::from("<logo@exploitx>".to_string()))
      .header(ContentDisposition::inline())
      .body(Body::new(content::LOGO_PNG.to_vec()))
  }

  fn mascot_part() -> SinglePart {
    SinglePart::builder()
      .header(
        ContentType::parse("image/png; name=\"mascot.png\"")
          .expect("invalid MIME type constant"),
      )
      .header(ContentId::from("<mascot@exploitx>".to_string()))
      .header(ContentDisposition::inline())
      .body(Body::new(content::MASCOT_PNG.to_vec()))
  }

  fn related(html: String) -> MultiPart {
    MultiPart::related()
      .singlepart(Self::html_part(html))
      .singlepart(Self::logo_part())
      .singlepart(Self::mascot_part())
  }
}

impl EmailService for GoogleEmailClient {
  async fn send_otp(
    &self,
    to: &Email,
    otp: &OtpCode,
    ttl_secs: i64,
  ) -> Result<(), EmailError> {
    let html = content::render_otp_email(otp.as_str(), ttl_secs);
    let email = Message::builder()
      .from(Mailbox::new(
        Some(self.from_name.clone()),
        self
          .from_email
          .parse::<Address>()
          .map_err(|e| EmailError::Template(e.to_string()))?,
      ))
      .to(Mailbox::new(
        None,
        to.as_str()
          .parse::<Address>()
          .map_err(|e| EmailError::Template(e.to_string()))?,
      ))
      .subject("Your Verification Code for Into The Void 2.0".to_string())
      .multipart(Self::related(html))
      .map_err(|e| EmailError::Template(e.to_string()))?;

    self
      .mailer
      .send(email)
      .await
      .map_err(|e| EmailError::Transport(e.to_string()))?;

    Ok(())
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
    let inner = Self::related(html);

    let email = if let Some(pdf_bytes) = pdf_data {
      let pdf_part = SinglePart::builder()
        .header(
          ContentType::parse("application/pdf")
            .expect("invalid MIME type constant"),
        )
        .header(lettre::message::header::ContentDisposition::attachment(
          &format!("invoice_{invoice_no}.pdf"),
        ))
        .body(Body::new(pdf_bytes));

      Message::builder()
        .from(Mailbox::new(
          Some(self.from_name.clone()),
          self
            .from_email
            .parse::<Address>()
            .map_err(|e| EmailError::Template(e.to_string()))?,
        ))
        .to(Mailbox::new(
          None,
          to.as_str()
            .parse::<Address>()
            .map_err(|e| EmailError::Template(e.to_string()))?,
        ))
        .subject("Registration Confirmed – Into The Void 2.0".to_string())
        .multipart(MultiPart::mixed().multipart(inner).singlepart(pdf_part))
        .map_err(|e| EmailError::Template(e.to_string()))?
    } else {
      Message::builder()
        .from(Mailbox::new(
          Some(self.from_name.clone()),
          self
            .from_email
            .parse::<Address>()
            .map_err(|e| EmailError::Template(e.to_string()))?,
        ))
        .to(Mailbox::new(
          None,
          to.as_str()
            .parse::<Address>()
            .map_err(|e| EmailError::Template(e.to_string()))?,
        ))
        .subject("Registration Confirmed – Into The Void 2.0".to_string())
        .multipart(inner)
        .map_err(|e| EmailError::Template(e.to_string()))?
    };

    self
      .mailer
      .send(email)
      .await
      .map_err(|e| EmailError::Transport(e.to_string()))?;

    Ok(())
  }
}
