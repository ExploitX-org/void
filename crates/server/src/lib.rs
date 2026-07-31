use axum::Router;
use axum::response::Redirect;
use database::{
  PgPaymentRepository, PgTeamRepository, PgVerificationRepository,
};
use email::{EmailService, GoogleEmailClient, PlunkEmailClient};
use payment::{CashfreeClient, PaymentGateway, RazorpayClient};
use sqlx::PgPool;

pub mod config;
pub mod crypto;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod services;
pub mod signing;
pub mod telemetry;
pub mod types;

#[derive(Clone)]
pub enum EmailClient {
  Google(GoogleEmailClient),
  Plunk(PlunkEmailClient),
}

impl EmailClient {
  pub async fn send_otp(
    &self,
    to: &domain::Email,
    otp: &domain::OtpCode,
    ttl_secs: i64,
  ) -> Result<(), email::EmailError> {
    match self {
      Self::Google(c) => c.send_otp(to, otp, ttl_secs).await,
      Self::Plunk(c) => c.send_otp(to, otp, ttl_secs).await,
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn send_invoice(
    &self,
    to: &domain::Email,
    leader_name: &str,
    invoice_no: &str,
    team_name: &str,
    reg_type: &str,
    amount: i32,
    pdf_data: Option<Vec<u8>>,
  ) -> Result<(), email::EmailError> {
    match self {
      Self::Google(c) => {
        c.send_invoice(
          to,
          leader_name,
          invoice_no,
          team_name,
          reg_type,
          amount,
          pdf_data,
        )
        .await
      }
      Self::Plunk(c) => {
        c.send_invoice(
          to,
          leader_name,
          invoice_no,
          team_name,
          reg_type,
          amount,
          pdf_data,
        )
        .await
      }
    }
  }
}

#[derive(Clone)]
pub enum PaymentClient {
  Cashfree(CashfreeClient),
  Razorpay(RazorpayClient),
}

impl PaymentClient {
  pub async fn create(
    &self,
    order_id: &str,
    amount: domain::Amount,
    customer: payment::PaymentCustomerInfo,
    callback_url: &str,
  ) -> Result<payment::PaymentCreateResponse, payment::PaymentError> {
    match self {
      Self::Cashfree(c) => {
        c.create(order_id, amount, customer, callback_url).await
      }
      Self::Razorpay(c) => {
        c.create(order_id, amount, customer, callback_url).await
      }
    }
  }

  pub async fn status(
    &self,
    order_id: &str,
  ) -> Result<payment::PaymentStatusResponse, payment::PaymentError> {
    match self {
      Self::Cashfree(c) => c.status(order_id).await,
      Self::Razorpay(c) => c.status(order_id).await,
    }
  }
}

#[derive(Clone)]
pub struct AppState {
  pub config: config::Config,
  pub pool: PgPool,
  pub team_repo: PgTeamRepository,
  pub verification_repo: PgVerificationRepository,
  pub payment_repo: PgPaymentRepository,
  pub email_client: EmailClient,
  pub payment_client: PaymentClient,
  pub verification: signing::VerificationConfig,
  pub nonce_cache: std::sync::Arc<std::sync::Mutex<signing::NonceCache>>,
}

pub fn build_router(state: AppState) -> Router {
  use axum::routing::{get, post};

  let mut router = Router::new()
    .route(
      routes::constants::REGISTER,
      post(handlers::register::register),
    )
    .route(routes::constants::VERIFY, post(handlers::verify::verify))
    .route(routes::constants::RESEND, post(handlers::verify::resend))
    .route(
      routes::constants::CREATE_PAYMENT,
      post(handlers::payment::create_payment),
    )
    .route(
      routes::constants::PAYMENT_STATUS,
      post(handlers::payment::payment_status),
    )
    .route(routes::constants::HEALTH, get(handlers::health::health))
    .route(
      "/health/",
      get(|| async { Redirect::permanent(routes::constants::HEALTH) }),
    );

  if state.config.is_sandbox {
    let sandbox_routes = Router::new()
      .route(
        routes::constants::SANDBOX,
        get(handlers::sandbox::get_sandbox),
      )
      .route(
        "/sandbox/openapi.json",
        get(handlers::sandbox::get_openapi_json),
      )
      .route(
        "/sandbox/",
        get(|| async { Redirect::permanent(routes::constants::SANDBOX) }),
      );

    router = router.merge(sandbox_routes);
  }

  router.with_state(state)
}
