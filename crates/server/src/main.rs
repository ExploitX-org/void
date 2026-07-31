use database::{
  PgPaymentRepository, PgTeamRepository, PgVerificationRepository, create_pool,
};
use email::{GoogleEmailClient, PlunkEmailClient};
use payment::{CashfreeClient, RazorpayClient};
use server::config::Config;
use server::{AppState, EmailClient, PaymentClient};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
  dotenvy::dotenv().ok();
  tracing_subscriber::fmt()
    .with_env_filter(
      EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info")),
    )
    .init();

  if let Err(e) = std::fs::write("version", env!("CARGO_PKG_VERSION")) {
    tracing::warn!(target: "void_srv::server", "failed to write version file: {e}");
  }

  let config = Config::from_env();
  tracing::info!(target: "void_srv::server", "config loaded");
  let host = config.host.clone();
  let port = config.port;

  let pool = create_pool(&config.database_url)
    .await
    .expect("failed to create database pool");
  tracing::info!(target: "void_srv::server", "database connected");

  let team_repo = PgTeamRepository::new(pool.clone());
  let verification_repo = PgVerificationRepository::new(pool.clone());
  let payment_repo = PgPaymentRepository::new(pool.clone());

  let email_client = match &config.email_provider {
    server::config::EmailProviderConfig::Google {
      smtp_host,
      smtp_port,
      smtp_secure,
      smtp_user,
      smtp_pass,
      from_name,
      from_email,
    } => EmailClient::Google(
      GoogleEmailClient::new(
        smtp_host.clone(),
        *smtp_port,
        *smtp_secure,
        smtp_user.clone(),
        smtp_pass.clone(),
        from_name.clone(),
        from_email.clone(),
      )
      .expect("failed to create Google email client"),
    ),
    server::config::EmailProviderConfig::Plunk {
      secret_key,
      from_email,
      from_name,
    } => EmailClient::Plunk(
      PlunkEmailClient::new(
        secret_key.clone(),
        from_email.clone(),
        from_name.clone(),
      )
      .expect("failed to create Plunk email client"),
    ),
  };

  let payment_client = match &config.payment_provider {
    server::config::PaymentProviderConfig::Cashfree(cfg) => {
      let is_production = cfg.environment == "production";
      PaymentClient::Cashfree(
        CashfreeClient::new(
          cfg.api_key.clone(),
          cfg.api_secret.clone(),
          is_production,
        )
        .expect("failed to create Cashfree client"),
      )
    }
    server::config::PaymentProviderConfig::Razorpay(cfg) => {
      PaymentClient::Razorpay(
        RazorpayClient::new(cfg.api_key.clone(), cfg.api_secret.clone())
          .expect("failed to create Razorpay client"),
      )
    }
  };

  let origins: Vec<axum::http::HeaderValue> = config
    .allowed_origins
    .iter()
    .map(|o| o.parse().expect("invalid origin in ALLOWED_ORIGINS"))
    .collect();

  let verification = server::signing::VerificationConfig::from_env();
  let nonce_cache = server::signing::NonceCache::new(
    verification.nonce_cache_max,
    Duration::from_secs(verification.timestamp_ttl_secs * 2),
  );
  let nonce_cache = Arc::new(std::sync::Mutex::new(nonce_cache));

  // Spawn nonce cache cleanup loop
  {
    let cache = nonce_cache.clone();
    let cleanup_interval = Duration::from_secs(30);
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(cleanup_interval);
      loop {
        interval.tick().await;
        cache.lock().expect("nonce cache lock poisoned").cleanup();
      }
    });
  }

  let state = AppState {
    config,
    pool: pool.clone(),
    team_repo,
    verification_repo,
    payment_repo,
    email_client,
    payment_client,
    verification,
    nonce_cache,
  };

  let cors = CorsLayer::new()
    .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
    .allow_origin(tower_http::cors::AllowOrigin::list(origins))
    .allow_headers([
      axum::http::header::CONTENT_TYPE,
      axum::http::header::ACCEPT,
    ]);

  let app = server::build_router(state.clone())
    .layer(RequestBodyLimitLayer::new(16_384))
    .layer(axum::middleware::from_fn_with_state(
      state,
      server::middleware::require_auth,
    ))
    .layer(axum::middleware::from_fn(server::telemetry::log_request))
    .layer(cors);

  let addr = format!("{}:{}", host, port);
  tracing::info!(target: "void_srv::server", addr = %addr, "listening");

  let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .expect("failed to bind address");

  axum::serve(listener, app).await.expect("server error");
}
