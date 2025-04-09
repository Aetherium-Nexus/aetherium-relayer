use ethers::providers::ProviderError;
use aetherium_core::ChainCommunicationError;

/// Errors from the crates specific to the aetherium-ethereum
/// implementation.
/// This error can then be converted into the broader error type
/// in aetherium-core using the `From` trait impl
#[derive(Debug, thiserror::Error)]
pub enum AetheriumEthereumError {
    /// provider Error
    #[error("{0}")]
    ProviderError(#[from] ProviderError),

    /// multicall Error
    #[error("Multicall contract error: {0}")]
    MulticallError(String),

    /// Some details from a queried block are missing
    #[error("Some details from a queried block are missing")]
    MissingBlockDetails,
}

impl From<AetheriumEthereumError> for ChainCommunicationError {
    fn from(value: AetheriumEthereumError) -> Self {
        ChainCommunicationError::from_other(value)
    }
}
