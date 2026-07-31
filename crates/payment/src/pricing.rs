use chrono::{DateTime, FixedOffset};
use domain::{Amount, DiscountInfo, Email, ParseError, RegistrationType};

#[derive(Debug, Clone)]
pub struct PricingService {
  portal_start: DateTime<FixedOffset>,
  base_price_paise: i32,
}

impl PricingService {
  pub fn new(
    portal_start: DateTime<FixedOffset>,
    base_price_paise: i32,
  ) -> Self {
    Self {
      portal_start,
      base_price_paise,
    }
  }

  pub fn calculate_amount(
    &self,
    registration_type: RegistrationType,
    leader_email: &Email,
    member_email: Option<&Email>,
  ) -> std::result::Result<(Amount, DiscountInfo), ParseError> {
    let eligible =
      self.is_eligible(registration_type, leader_email, member_email);
    let base = self.base_price_paise;

    let (discount_amount, coupon_code): (i32, Option<String>) = if eligible {
      let now = chrono::Utc::now().with_timezone(self.portal_start.offset());
      let days_since = now
        .date_naive()
        .signed_duration_since(self.portal_start.date_naive())
        .num_days();

      if days_since < 0 {
        (0, None)
      } else if days_since < 7 {
        (base / 2, Some("ZERODAY".into()))
      } else if days_since < 14 {
        (base / 4, Some("FLAGFOUND".into()))
      } else {
        (0, None)
      }
    } else {
      (0, None)
    };

    let final_ = base.saturating_sub(discount_amount);
    let amount = Amount::parse(final_)?;
    let info = DiscountInfo {
      base_price_paise: base,
      final_price_paise: final_,
      discount_amount,
      coupon_code,
    };
    Ok((amount, info))
  }

  fn is_eligible(
    &self,
    reg_type: RegistrationType,
    leader_email: &Email,
    member_email: Option<&Email>,
  ) -> bool {
    let leader_eligible = leader_email
      .domain()
      .is_some_and(|d| d.eq_ignore_ascii_case("citchennai.net"));
    match reg_type {
      RegistrationType::Single => leader_eligible,
      RegistrationType::Couple => {
        leader_eligible
          && member_email
            .and_then(|m| m.domain())
            .is_some_and(|d| d.eq_ignore_ascii_case("citchennai.net"))
      }
    }
  }
}
