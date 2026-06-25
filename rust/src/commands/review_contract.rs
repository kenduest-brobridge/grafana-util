//! Shared review/action contract vocabulary.
//!
//! Keep machine-readable action and status strings centralized so plan, preview,
//! apply, and TUI layers do not drift when comparing the same review contract.

#[path = "review_contract/actions.rs"]
mod actions;
#[cfg(any(feature = "tui", test))]
#[path = "review_contract/detail.rs"]
mod detail;
#[path = "review_contract/envelope.rs"]
mod envelope;
#[path = "review_contract/model.rs"]
mod model;

pub(crate) use actions::*;
#[cfg(any(feature = "tui", test))]
pub(crate) use detail::*;
pub(crate) use envelope::*;
pub(crate) use model::*;

#[cfg(test)]
#[path = "review_contract_tests.rs"]
mod tests;
