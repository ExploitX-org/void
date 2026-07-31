use std::collections::HashMap;
use std::env;
use std::fmt;

use chrono::{DateTime, FixedOffset};
use invoice::PdfMethod;

#[derive(Clone)]
pub struct Config {
  pub host: String,
  pub port: u16,
  pub database_url: String,
  pub portal_start: DateTime<FixedOffset>,
  pub base_price_paise: i32,
  pub otp_ttl_secs: i64,
  pub otp_max_attempts: u32,
  pub max_resend_otp_retry_attempts: u32,
  pub auth_hmac_secret_for_health: String,
  pub auth_hmac_secret_for_register: String,
  pub auth_hmac_secret_for_authn_verify: String,
  pub auth_hmac_secret_for_authn_resend: String,
  pub auth_hmac_secret_for_pmt_create: String,
  pub auth_hmac_secret_for_pmt_status: String,
  pub auth_hmac_secret_for_sandbox: String,

  pub email_provider: EmailProviderConfig,
  pub payment_provider: PaymentProviderConfig,
  pub invoice_method: PdfMethod,
  pub assets_dir: String,

  pub frontend_url: String,
  pub allowed_origins: Vec<String>,
  pub payment_callback_url_path: String,

  pub is_sandbox: bool,
  pub backend_url: String,

  pub team_name_min_len: usize,
  pub team_name_max_len: usize,
  pub mobile_cc_min_len: usize,
  pub mobile_cc_max_len: usize,
  pub mobile_number_min_len: usize,
  pub mobile_number_max_len: usize,

  pub signing_timestamp_ttl_secs: u64,
  pub signing_nonce_cache_max: usize,
}

#[derive(Clone)]
pub enum EmailProviderConfig {
  Google {
    smtp_host: String,
    smtp_port: u16,
    smtp_secure: bool,
    smtp_user: String,
    smtp_pass: String,
    from_name: String,
    from_email: String,
  },
  Plunk {
    secret_key: String,
    from_email: String,
    from_name: String,
  },
}

#[derive(Clone)]
pub struct CashfreeConfig {
  pub api_key: String,
  pub api_secret: String,
  pub environment: String,
}

#[derive(Clone)]
pub struct RazorpayConfig {
  pub api_key: String,
  pub api_secret: String,
  #[allow(dead_code)]
  pub webhook_secret: String,
}

#[derive(Clone)]
pub enum PaymentProviderConfig {
  Cashfree(CashfreeConfig),
  Razorpay(RazorpayConfig),
}

impl Config {
  pub fn from_env() -> Self {
    let portal_start_raw = env::var("PORTAL_START_TIMESTAMP")
      .unwrap_or_else(|_| "2026-07-11T00:00:00+05:30".into());
    let portal_start: DateTime<FixedOffset> = portal_start_raw
      .trim()
      .parse()
      .expect("PORTAL_START_TIMESTAMP must be a valid RFC 3339 timestamp");

    let base_price_raw = env::var("CTF_BASE_REGISTRATION_PRICE_INR")
      .unwrap_or_else(|_| "200".into());
    let base_price_paise = base_price_raw
      .trim()
      .parse::<i32>()
      .expect("CTF_BASE_REGISTRATION_PRICE_INR must be a valid integer")
      * 100;

    let use_typst_from =
      env::var("USE_TYPST_FROM").unwrap_or_else(|_| "crate".into());
    let invoice_method = match use_typst_from.trim() {
      "crate" => PdfMethod::Crate,
      _ => PdfMethod::Cli,
    };

    let email_service_provider =
      env::var("EMAIL_SERVICE_PROVIDER").unwrap_or_else(|_| "google".into());
    let email_provider = match email_service_provider.trim() {
      "plunk" => EmailProviderConfig::Plunk {
        secret_key: env::var("PLUNK_SECRET_API_KEY")
          .expect(
            "PLUNK_SECRET_API_KEY required when EMAIL_SERVICE_PROVIDER=plunk",
          )
          .trim()
          .to_string(),
        from_email: env::var("PLUNK_FROM_EMAIL")
          .expect("PLUNK_FROM_EMAIL required")
          .trim()
          .to_string(),
        from_name: env::var("EMAIL_SUBJECT")
          .unwrap_or_else(|_| "void-srv".into())
          .trim()
          .to_string(),
      },
      _ => EmailProviderConfig::Google {
        smtp_host: env::var("SMTP_HOST")
          .unwrap_or_else(|_| "smtp.gmail.com".into())
          .trim()
          .to_string(),
        smtp_port: env::var("SMTP_PORT")
          .unwrap_or_else(|_| "587".into())
          .trim()
          .parse()
          .expect("SMTP_PORT must be a valid port"),
        smtp_secure: env::var("SMTP_SECURE")
          .unwrap_or_else(|_| "false".into())
          .trim()
          .parse::<bool>()
          .expect("SMTP_SECURE must be a boolean"),
        smtp_user: env::var("SMTP_USER")
          .expect("SMTP_USER required")
          .trim()
          .to_string(),
        smtp_pass: env::var("SMTP_PASS")
          .expect("SMTP_PASS required")
          .trim()
          .to_string(),
        from_name: env::var("EMAIL_SUBJECT")
          .unwrap_or_else(|_| "void-srv".into())
          .trim()
          .to_string(),
        from_email: env::var("GOOGLE_FROM_EMAIL")
          .expect("GOOGLE_FROM_EMAIL required")
          .trim()
          .to_string(),
      },
    };

    let payment_gateway_provider = env::var("PAYMENT_GATEWAY_PROVIDER")
      .unwrap_or_else(|_| "razorpay".into());
    let payment_provider = match payment_gateway_provider.trim() {
      "cashfree" => PaymentProviderConfig::Cashfree(CashfreeConfig {
        api_key: env::var("CASHFREE_API_ID")
          .expect("CASHFREE_API_ID required")
          .trim()
          .to_string(),
        api_secret: env::var("CASHFREE_API_SECRET")
          .expect("CASHFREE_API_SECRET required")
          .trim()
          .to_string(),
        environment: env::var("CASHFREE_ENV")
          .unwrap_or_else(|_| "sandbox".into())
          .trim()
          .to_string(),
      }),
      _ => PaymentProviderConfig::Razorpay(RazorpayConfig {
        api_key: env::var("RAZORPAY_API_KEY")
          .expect("RAZORPAY_API_KEY required")
          .trim()
          .to_string(),
        api_secret: env::var("RAZORPAY_KEY_SECRET")
          .expect("RAZORPAY_KEY_SECRET required")
          .trim()
          .to_string(),
        webhook_secret: env::var("RZP_WEBHOOK_SECRET")
          .unwrap_or_else(|_| "".into())
          .trim()
          .to_string(),
      }),
    };

    let assets_dir_raw =
      env::var("ASSETS_DIR").unwrap_or_else(|_| "./assets".into());
    let assets_dir = assets_dir_raw.trim().trim_end_matches('/').to_string();
    if assets_dir.starts_with('/') {
      panic!("ASSETS_DIR must be a relative path, got: {assets_dir}");
    }

    let host = env::var("HOST")
      .unwrap_or_else(|_| "0.0.0.0".into())
      .trim()
      .to_string();
    let port: u16 = env::var("PORT")
      .unwrap_or_else(|_| "3000".into())
      .trim()
      .parse()
      .expect("PORT must be a valid port");

    let config = Self {
      host: host.clone(),
      port,
      frontend_url: env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:5000".into())
        .trim()
        .trim_end_matches('/')
        .to_string(),
      allowed_origins: env::var("ALLOWED_ORIGINS")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| {
          s.split(',')
            .map(|o| o.trim().trim_end_matches('/').to_string())
            .filter(|o| !o.is_empty())
            .collect()
        })
        .unwrap_or_else(|| {
          vec![
            env::var("FRONTEND_URL")
              .unwrap_or_else(|_| "http://localhost:5000".into())
              .trim()
              .trim_end_matches('/')
              .to_string(),
          ]
        }),
      database_url: env::var("DATABASE_URL")
        .expect("DATABASE_URL required")
        .trim()
        .to_string(),
      portal_start,
      base_price_paise,
      otp_ttl_secs: env::var("OTP_TTL_SECS")
        .unwrap_or_else(|_| "300".into())
        .trim()
        .parse()
        .expect("OTP_TTL_SECS must be a valid integer"),
      otp_max_attempts: env::var("MAX_OTP_RETRY_ATTEMPTS")
        .unwrap_or_else(|_| "3".into())
        .trim()
        .parse()
        .expect("MAX_OTP_RETRY_ATTEMPTS must be a valid integer"),
      max_resend_otp_retry_attempts: env::var("MAX_RESEND_OTP_RETRY_ATTEMPTS")
        .unwrap_or_else(|_| "3".into())
        .trim()
        .parse()
        .expect("MAX_RESEND_OTP_RETRY_ATTEMPTS must be a valid integer"),
      auth_hmac_secret_for_health: env::var(
        "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_HEALTH",
      )
      .expect("ADMIN_AUTH_HMAC_SECRET_KEY_FOR_HEALTH required")
      .trim()
      .to_string(),
      auth_hmac_secret_for_register: env::var(
        "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_REGISTER",
      )
      .expect("ADMIN_AUTH_HMAC_SECRET_KEY_FOR_REGISTER required")
      .trim()
      .to_string(),
      auth_hmac_secret_for_authn_verify: env::var(
        "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_AUTHN_VERIFY",
      )
      .expect("ADMIN_AUTH_HMAC_SECRET_KEY_FOR_AUTHN_VERIFY required")
      .trim()
      .to_string(),
      auth_hmac_secret_for_authn_resend: env::var(
        "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_AUTHN_RESEND",
      )
      .expect("ADMIN_AUTH_HMAC_SECRET_KEY_FOR_AUTHN_RESEND required")
      .trim()
      .to_string(),
      auth_hmac_secret_for_pmt_create: env::var(
        "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_PMT_CREATE",
      )
      .expect("ADMIN_AUTH_HMAC_SECRET_KEY_FOR_PMT_CREATE required")
      .trim()
      .to_string(),
      auth_hmac_secret_for_pmt_status: env::var(
        "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_PMT_STATUS",
      )
      .expect("ADMIN_AUTH_HMAC_SECRET_KEY_FOR_PMT_STATUS required")
      .trim()
      .to_string(),
      auth_hmac_secret_for_sandbox: env::var(
        "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_SANDBOX",
      )
      .expect("ADMIN_AUTH_HMAC_SECRET_KEY_FOR_SANDBOX required")
      .trim()
      .to_string(),
      email_provider,
      payment_provider,
      invoice_method,
      assets_dir,
      is_sandbox: env::var("IS_SANDBOX")
        .unwrap_or_else(|_| "false".into())
        .trim()
        == "true",
      backend_url: env::var("BACKEND_URL")
        .unwrap_or_else(|_| format!("http://{host}:{port}"))
        .trim()
        .trim_end_matches('/')
        .to_string(),
      payment_callback_url_path: format!(
        "/{}",
        env::var("PAYMENT_GATEWAY_CALLBACK_URL_PATH")
          .unwrap_or_else(|_| "payment/status".into())
          .trim()
          .trim_matches('/')
      ),
      team_name_min_len: env::var("TEAM_NAME_MIN_LEN")
        .unwrap_or_else(|_| "3".into())
        .trim()
        .parse()
        .expect("TEAM_NAME_MIN_LEN must be a valid usize"),
      team_name_max_len: env::var("TEAM_NAME_MAX_LEN")
        .unwrap_or_else(|_| "25".into())
        .trim()
        .parse()
        .expect("TEAM_NAME_MAX_LEN must be a valid usize"),
      mobile_cc_min_len: env::var("MOBILE_CC_MIN_LEN")
        .unwrap_or_else(|_| "2".into())
        .trim()
        .parse()
        .expect("MOBILE_CC_MIN_LEN must be a valid usize"),
      mobile_cc_max_len: env::var("MOBILE_CC_MAX_LEN")
        .unwrap_or_else(|_| "4".into())
        .trim()
        .parse()
        .expect("MOBILE_CC_MAX_LEN must be a valid usize"),
      mobile_number_min_len: env::var("MOBILE_NUMBER_MIN_LEN")
        .unwrap_or_else(|_| "10".into())
        .trim()
        .parse()
        .expect("MOBILE_NUMBER_MIN_LEN must be a valid usize"),
      mobile_number_max_len: env::var("MOBILE_NUMBER_MAX_LEN")
        .unwrap_or_else(|_| "10".into())
        .trim()
        .parse()
        .expect("MOBILE_NUMBER_MAX_LEN must be a valid usize"),
      signing_timestamp_ttl_secs: env::var("SIGNING_TIMESTAMP_TTL_SECS")
        .unwrap_or_else(|_| "60".into())
        .trim()
        .parse()
        .expect("SIGNING_TIMESTAMP_TTL_SECS must be a valid integer"),
      signing_nonce_cache_max: env::var("SIGNING_NONCE_CACHE_MAX")
        .unwrap_or_else(|_| "500000".into())
        .trim()
        .parse()
        .expect("SIGNING_NONCE_CACHE_MAX must be a valid integer"),
    };
    validate_hmac_secrets(&config);
    config
  }
}

fn validate_hmac_secrets(config: &Config) {
  let secrets = [
    (
      "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_HEALTH",
      &config.auth_hmac_secret_for_health,
    ),
    (
      "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_REGISTER",
      &config.auth_hmac_secret_for_register,
    ),
    (
      "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_AUTHN_VERIFY",
      &config.auth_hmac_secret_for_authn_verify,
    ),
    (
      "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_AUTHN_RESEND",
      &config.auth_hmac_secret_for_authn_resend,
    ),
    (
      "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_PMT_CREATE",
      &config.auth_hmac_secret_for_pmt_create,
    ),
    (
      "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_PMT_STATUS",
      &config.auth_hmac_secret_for_pmt_status,
    ),
    (
      "ADMIN_AUTH_HMAC_SECRET_KEY_FOR_SANDBOX",
      &config.auth_hmac_secret_for_sandbox,
    ),
  ];

  for (key, val) in &secrets {
    if val.is_empty() {
      panic!("{key} must not be empty");
    }
    if val.len() < 256 {
      panic!("{key} must be at least 256 characters, got {}", val.len());
    }
  }

  let mut seen: HashMap<&str, Vec<&str>> = HashMap::new();
  for (key, val) in &secrets {
    seen.entry(val.as_str()).or_default().push(key);
  }
  let dupes: Vec<&str> = seen
    .into_values()
    .filter(|v| v.len() > 1)
    .flatten()
    .collect();
  if !dupes.is_empty() {
    panic!("duplicate HMAC secrets: {}", dupes.join(", "));
  }
}

impl fmt::Debug for Config {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Config")
      .field("host", &self.host)
      .field("port", &self.port)
      .field("database_url", &"***")
      .field("portal_start", &self.portal_start)
      .field("base_price_paise", &self.base_price_paise)
      .field("otp_ttl_secs", &self.otp_ttl_secs)
      .field("otp_max_attempts", &self.otp_max_attempts)
      .field(
        "max_resend_otp_retry_attempts",
        &self.max_resend_otp_retry_attempts,
      )
      .field("auth_hmac_secret_for_health", &"***")
      .field("auth_hmac_secret_for_register", &"***")
      .field("auth_hmac_secret_for_authn_verify", &"***")
      .field("auth_hmac_secret_for_authn_resend", &"***")
      .field("auth_hmac_secret_for_pmt_create", &"***")
      .field("auth_hmac_secret_for_pmt_status", &"***")
      .field("auth_hmac_secret_for_sandbox", &"***")
      .field("email_provider", &self.email_provider)
      .field("payment_provider", &self.payment_provider)
      .field("invoice_method", &self.invoice_method)
      .field("assets_dir", &self.assets_dir)
      .field("frontend_url", &self.frontend_url)
      .field("allowed_origins", &self.allowed_origins)
      .field("payment_callback_url_path", &self.payment_callback_url_path)
      .field("is_sandbox", &self.is_sandbox)
      .field("backend_url", &self.backend_url)
      .field("team_name_min_len", &self.team_name_min_len)
      .field("team_name_max_len", &self.team_name_max_len)
      .field("mobile_cc_min_len", &self.mobile_cc_min_len)
      .field("mobile_cc_max_len", &self.mobile_cc_max_len)
      .field("mobile_number_min_len", &self.mobile_number_min_len)
      .field("mobile_number_max_len", &self.mobile_number_max_len)
      .finish()
  }
}

impl fmt::Debug for EmailProviderConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Google {
        smtp_host,
        smtp_port,
        smtp_secure,
        from_name,
        from_email,
        ..
      } => f
        .debug_struct("Google")
        .field("smtp_host", smtp_host)
        .field("smtp_port", smtp_port)
        .field("smtp_secure", smtp_secure)
        .field("smtp_user", &"***")
        .field("smtp_pass", &"***")
        .field("from_name", from_name)
        .field("from_email", from_email)
        .finish(),
      Self::Plunk {
        from_email,
        from_name,
        ..
      } => f
        .debug_struct("Plunk")
        .field("secret_key", &"***")
        .field("from_email", from_email)
        .field("from_name", from_name)
        .finish(),
    }
  }
}

impl fmt::Debug for PaymentProviderConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Cashfree(c) => f.debug_tuple("Cashfree").field(c).finish(),
      Self::Razorpay(r) => f.debug_tuple("Razorpay").field(r).finish(),
    }
  }
}

impl fmt::Debug for CashfreeConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CashfreeConfig")
      .field("api_key", &"***")
      .field("api_secret", &"***")
      .field("environment", &self.environment)
      .finish()
  }
}

impl fmt::Debug for RazorpayConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RazorpayConfig")
      .field("api_key", &"***")
      .field("api_secret", &"***")
      .field("webhook_secret", &"***")
      .finish()
  }
}
