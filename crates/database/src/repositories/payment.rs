use domain::{
  PaymentProvider, PaymentRepository, PaymentRow, PaymentStatus,
  RepositoryError,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct PgPaymentRepository {
  pool: PgPool,
}

impl PgPaymentRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

impl PaymentRepository for PgPaymentRepository {
  async fn create_payment(
    &self,
    team_id: Uuid,
    provider: PaymentProvider,
    provider_order_id: &str,
    amount: i32,
    payment_link: &str,
  ) -> Result<Uuid, RepositoryError> {
    let provider_str = match provider {
      PaymentProvider::Cashfree => "cashfree",
      PaymentProvider::Razorpay => "razorpay",
    };

    let payment_id: Uuid = sqlx::query_scalar(
      r#"
            INSERT INTO payments (team_id, provider, provider_order_id, amount, payment_link, status)
            VALUES ($1, $2::payment_provider, $3, $4, $5, 'initiated')
            RETURNING id
            "#,
    )
    .bind(team_id)
    .bind(provider_str)
    .bind(provider_order_id)
    .bind(amount)
    .bind(payment_link)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(payment_id)
  }

  async fn find_payment_by_team_id(
    &self,
    team_id: Uuid,
  ) -> Result<Option<PaymentRow>, RepositoryError> {
    let row = sqlx::query(
      r#"
            SELECT id, provider::text, provider_order_id, amount, status::text, payment_link,
                   ordered_at, completed_at
            FROM payments
            WHERE team_id = $1
            ORDER BY ordered_at DESC
            LIMIT 1
            "#,
    )
    .bind(team_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?
    .map(|r| {
      Ok(PaymentRow {
        id: r.get("id"),
        provider: r.get("provider"),
        provider_order_id: r.get("provider_order_id"),
        amount: r.get("amount"),
        status: r.get("status"),
        payment_link: r.get("payment_link"),
        ordered_at: r.get("ordered_at"),
        completed_at: r.try_get("completed_at").ok().flatten(),
      })
    })
    .transpose()?;

    Ok(row)
  }

  async fn update_payment_status(
    &self,
    payment_id: Uuid,
    status: PaymentStatus,
  ) -> Result<(), RepositoryError> {
    let status_str = match status {
      PaymentStatus::Initiated => "initiated",
      PaymentStatus::Completed => "completed",
      PaymentStatus::Failed => "failed",
      PaymentStatus::Refunded => "refunded",
    };

    sqlx::query(
      r#"UPDATE payments SET status = $1::payment_status, completed_at = CASE WHEN $1::payment_status = 'completed' AND completed_at IS NULL THEN NOW() ELSE completed_at END WHERE id = $2"#,
    )
    .bind(status_str)
    .bind(payment_id)
    .execute(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(())
  }

  async fn next_invoice_number(&self) -> Result<i32, RepositoryError> {
    let seq: i32 = sqlx::query_scalar("SELECT nextval('invoice_seq')::int")
      .fetch_one(&self.pool)
      .await
      .map_err(|e| RepositoryError::Database(e.to_string()))?;
    Ok(seq)
  }

  async fn get_payment_amount(
    &self,
    team_id: Uuid,
  ) -> Result<i32, RepositoryError> {
    let amount: Option<i32> = sqlx::query_scalar(
      r#"
            SELECT amount FROM payments
            WHERE team_id = $1
            ORDER BY ordered_at DESC
            LIMIT 1
            "#,
    )
    .bind(team_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    amount.ok_or_else(|| RepositoryError::NotFound("payment for team".into()))
  }
}
