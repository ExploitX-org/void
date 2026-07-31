use domain::{
  Amount, InvoiceNo, PaymentProvider, PaymentRepository, PaymentStatus,
  TeamRepository, TeamState,
};

use payment::PaymentCustomerInfo;

use crate::AppState;
use crate::error::ApiError;
use crate::types::{ParsedCreatePaymentRequest, ParsedPaymentStatusRequest};

pub struct PaymentService;

impl PaymentService {
  pub async fn create(
    state: &AppState,
    req: ParsedCreatePaymentRequest,
  ) -> Result<String, ApiError> {
    let team = state.team_repo.find_by_id(req.team_id).await?;

    match team.state {
      TeamState::Verified | TeamState::PaymentPending => {}
      _ => {
        return Err(ApiError::BadRequest(
          "team must be verified before payment".into(),
        ));
      }
    }

    let final_price = Amount::parse(
      team
        .final_price_paise
        .or(team.base_price_paise)
        .unwrap_or(20000),
    )?;

    if let Some(ref existing) = state
      .payment_repo
      .find_payment_by_team_id(req.team_id)
      .await?
    {
      if existing.status == "completed" {
        Self::generate_invoice_for_team(
          state,
          &team,
          &existing.provider_order_id,
          &existing.provider,
          &existing.ordered_at,
        )
        .await;
        return Ok(existing.payment_link.clone());
      }

      match state
        .payment_client
        .status(&existing.provider_order_id)
        .await
      {
        Ok(gw_status)
          if gw_status.status.eq_ignore_ascii_case("ACTIVE")
            || gw_status.status.eq_ignore_ascii_case("PAID") =>
        {
          if gw_status.status.eq_ignore_ascii_case("PAID") {
            let changed = state
              .team_repo
              .update_team_state(req.team_id, TeamState::Paid)
              .await?;
            state
              .payment_repo
              .update_payment_status(existing.id, PaymentStatus::Completed)
              .await?;
            if changed {
              Self::generate_invoice_for_team(
                state,
                &team,
                &existing.provider_order_id,
                &existing.provider,
                &existing.ordered_at,
              )
              .await;
            }
          }
          return Ok(existing.payment_link.clone());
        }
        Ok(_) => {}
        Err(e) => {
          tracing::error!(
            "payment status check failed for {}: {e}",
            existing.provider_order_id
          );
          return Err(ApiError::Internal(
            "Failed to verify existing payment status.".into(),
          ));
        }
      }
    }

    let order_id = payment::generate_order_id();
    let customer = PaymentCustomerInfo {
      name: team.leader_name.clone(),
      email: team.leader_email.as_str().to_string(),
      phone: format!("{}{}", team.leader_mobile_cc, team.leader_mobile_number),
      team_name: team.team_name.clone(),
    };
    let callback_url = format!(
      "{}{}?team_id={}",
      state.config.frontend_url,
      state.config.payment_callback_url_path,
      req.team_id,
    );

    let order = state
      .payment_client
      .create(&order_id, final_price, customer, &callback_url)
      .await?;

    let provider = match order.provider.as_str() {
      "cashfree" => PaymentProvider::Cashfree,
      "razorpay" => PaymentProvider::Razorpay,
      other => {
        return Err(ApiError::Internal(format!(
          "unknown payment provider: {other}"
        )));
      }
    };

    state
      .payment_repo
      .create_payment(
        req.team_id,
        provider,
        &order.order_id,
        final_price.get(),
        &order.payment_link,
      )
      .await?;

    state
      .team_repo
      .update_team_state(req.team_id, TeamState::PaymentPending)
      .await?;

    Ok(order.payment_link)
  }

  async fn generate_invoice_for_team(
    state: &AppState,
    team: &domain::TeamDetails,
    order_id: &str,
    provider: &str,
    ordered_at: &chrono::DateTime<chrono::Utc>,
  ) {
    let seq = match state.payment_repo.next_invoice_number().await {
      Ok(n) => n,
      Err(e) => {
        tracing::error!("failed to get invoice sequence: {e}");
        return;
      }
    };
    let invoice_no = match InvoiceNo::parse(format!("EX-INV-ITV2-{seq}")) {
      Ok(n) => n,
      Err(e) => {
        tracing::error!("invalid invoice number: {e}");
        return;
      }
    };

    let ist = match chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60) {
      Some(offset) => offset,
      None => {
        tracing::error!("invalid IST offset computed");
        return;
      }
    };
    let fmt_dt = |dt: &chrono::DateTime<chrono::Utc>| {
      dt.with_timezone(&ist)
        .format("%I:%M:%S%.3f %p IST, %B %-d, %Y")
        .to_string()
    };

    let base_rupees = team.base_price_paise.unwrap_or(20000) / 100;
    let final_rupees = team.final_price_paise.unwrap_or(0) / 100;
    let discount_rupees = base_rupees.saturating_sub(final_rupees);

    let discount_code = team.discount_code.as_deref();

    let payment_provider = match provider {
      "razorpay" => "Razorpay",
      _ => "Cashfree",
    };

    let invoice_data = invoice::build_invoice_data(
      invoice_no.as_str(),
      &team.leader_name,
      &team.leader_location,
      team.leader_email.as_str(),
      &fmt_dt(ordered_at),
      &fmt_dt(&chrono::Utc::now()),
      base_rupees,
      base_rupees,
      discount_rupees,
      discount_code,
      final_rupees,
      final_rupees,
      0,
      Some(order_id),
      payment_provider,
      order_id,
    );

    match invoice::generate_invoice(
      std::path::Path::new(&state.config.assets_dir),
      &invoice_data,
      state.config.invoice_method,
    ) {
      Ok(pdf_bytes) => {
        let has_pdf = !pdf_bytes.is_empty();
        let leader_email = &team.leader_email;
        let amount_rupees = team.final_price_paise.unwrap_or(0) / 100;
        let pdf_opt = if has_pdf { Some(pdf_bytes) } else { None };
        if let Err(e) = state
          .email_client
          .send_invoice(
            leader_email,
            &team.leader_name,
            invoice_no.as_str(),
            &team.team_name,
            team.registration_type.as_str(),
            amount_rupees,
            pdf_opt,
          )
          .await
        {
          tracing::error!("failed to send invoice email to leader: {e}");
        }

        if let Some(ref member_email) = team.member_email
          && let Err(e) = state
            .email_client
            .send_invoice(
              member_email,
              &team.leader_name,
              invoice_no.as_str(),
              &team.team_name,
              team.registration_type.as_str(),
              amount_rupees,
              None,
            )
            .await
        {
          tracing::error!("failed to send invoice email to member: {e}");
        }
      }
      Err(e) => {
        tracing::warn!("invoice generation failed: {e}");
      }
    }
  }

  pub async fn status(
    state: &AppState,
    req: ParsedPaymentStatusRequest,
  ) -> Result<crate::types::PaymentStatusData, ApiError> {
    let team = state.team_repo.find_by_id(req.team_id).await?;

    let payment = state
      .payment_repo
      .find_payment_by_team_id(req.team_id)
      .await?
      .ok_or_else(|| {
        ApiError::NotFound("no payment found for this team".into())
      })?;

    let gw_status = state
      .payment_client
      .status(&payment.provider_order_id)
      .await?;

    let is_success = gw_status.status.eq_ignore_ascii_case("PAID")
      || gw_status.status.eq_ignore_ascii_case("COMPLETED")
      || gw_status.status.eq_ignore_ascii_case("CAPTURED");

    if is_success && team.state != TeamState::Paid {
      let changed = state
        .team_repo
        .update_team_state(req.team_id, TeamState::Paid)
        .await?;

      state
        .payment_repo
        .update_payment_status(payment.id, PaymentStatus::Completed)
        .await?;

      if !changed {
        return Ok(crate::types::PaymentStatusData {
          status: gw_status.status,
          payment_provider: Some(payment.provider.clone()),
          payment_order_id: Some(payment.provider_order_id.clone()),
        });
      }

      Self::generate_invoice_for_team(
        state,
        &team,
        &payment.provider_order_id,
        &payment.provider,
        &payment.ordered_at,
      )
      .await;
    }

    let payment_order_id = Some(payment.provider_order_id.clone());

    Ok(crate::types::PaymentStatusData {
      status: gw_status.status,
      payment_provider: Some(payment.provider.clone()),
      payment_order_id,
    })
  }
}
