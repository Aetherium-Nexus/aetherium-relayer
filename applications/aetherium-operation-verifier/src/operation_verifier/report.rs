use aetherium_application::ApplicationReport;
use aetherium_core::{AetheriumMessage, U256};

/// Application operation verifier report
#[derive(Debug, Eq, PartialEq)]
pub enum ApplicationOperationVerifierReport {
    /// Amount below minimum (minimum, actual)
    AmountBelowMinimum { minimum: U256, actual: U256 },
    /// Message is malformed
    MalformedMessage(AetheriumMessage),
    /// Zero amount
    ZeroAmount,
}

impl From<&ApplicationOperationVerifierReport> for ApplicationReport {
    fn from(value: &ApplicationOperationVerifierReport) -> ApplicationReport {
        use crate::ApplicationOperationVerifierReport::*;

        match value {
            AmountBelowMinimum {
                minimum: _,
                actual: _,
            } => ApplicationReport::AmountBelowMinimum,
            MalformedMessage(_) => ApplicationReport::MalformedMessage,
            ZeroAmount => ApplicationReport::ZeroAmount,
        }
    }
}

impl From<ApplicationOperationVerifierReport> for ApplicationReport {
    fn from(value: ApplicationOperationVerifierReport) -> ApplicationReport {
        ApplicationReport::from(&value)
    }
}
