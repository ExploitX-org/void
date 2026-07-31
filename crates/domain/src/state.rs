use serde::{Deserialize, Serialize};

use crate::enums::TeamState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidTransition {
  pub from: TeamState,
  pub to: TeamState,
}

impl std::fmt::Display for InvalidTransition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "cannot transition from {:?} to {:?}", self.from, self.to)
  }
}

impl std::error::Error for InvalidTransition {}

impl TeamState {
  pub fn can_transition_to(&self, next: TeamState) -> bool {
    matches!(
      (self, next),
      (Self::PendingVerification, Self::Verified)
        | (Self::PendingVerification, Self::Failed)
        | (Self::Verified, Self::PaymentPending)
        | (Self::Verified, Self::Failed)
        | (Self::PaymentPending, Self::Paid)
        | (Self::PaymentPending, Self::Failed)
        | (Self::Failed, Self::PendingVerification)
    )
  }

  pub fn transition(
    &self,
    next: TeamState,
  ) -> std::result::Result<TeamState, InvalidTransition> {
    if self.can_transition_to(next) {
      Ok(next)
    } else {
      Err(InvalidTransition {
        from: *self,
        to: next,
      })
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn valid_transitions_all_succeed() {
    let cases = [
      (TeamState::PendingVerification, TeamState::Verified, true),
      (TeamState::PendingVerification, TeamState::Failed, true),
      (TeamState::Verified, TeamState::PaymentPending, true),
      (TeamState::Verified, TeamState::Failed, true),
      (TeamState::PaymentPending, TeamState::Paid, true),
      (TeamState::PaymentPending, TeamState::Failed, true),
      (TeamState::Failed, TeamState::PendingVerification, true),
      (TeamState::Paid, TeamState::PendingVerification, false),
      (TeamState::Paid, TeamState::Failed, false),
      (TeamState::Verified, TeamState::PendingVerification, false),
      (TeamState::PendingVerification, TeamState::Paid, false),
    ];
    for (from, to, expected) in &cases {
      assert_eq!(
        from.can_transition_to(*to),
        *expected,
        "{:?} -> {:?}",
        from,
        to
      );
    }
  }

  #[test]
  fn transition_ok() {
    let result = TeamState::PendingVerification.transition(TeamState::Verified);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TeamState::Verified);
  }

  #[test]
  fn transition_err() {
    let result = TeamState::Paid.transition(TeamState::Failed);
    assert!(result.is_err());
  }
}
