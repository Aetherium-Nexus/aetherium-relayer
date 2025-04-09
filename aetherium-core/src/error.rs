use std::any::Any;
use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

use bigdecimal::ParseBigDecimalError;
use derive_new::new;

use crate::config::StrOrIntParseError;
use crate::rpc_clients::RpcClientError;
use std::string::FromUtf8Error;

use crate::{
    Error as PrimitiveTypeError, AetheriumProviderError, AetheriumSignerError, ReorgPeriod, H256,
    U256,
};

/// The result of interacting with a chain.
pub type ChainResult<T> = Result<T, ChainCommunicationError>;

/// An "Any"-typed error.
pub trait AetheriumCustomError: StdError + Send + Sync + Any {}

impl<E: StdError + Send + Sync + Any> AetheriumCustomError for E {}

/// Thin wrapper around a boxed AetheriumCustomError; required to satisfy
/// AsDynError implementations. Basically a trait-object adaptor.
#[repr(transparent)]
#[derive(new)]
pub struct AetheriumCustomErrorWrapper(Box<dyn AetheriumCustomError>);

impl Debug for AetheriumCustomErrorWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", AsRef::<dyn AetheriumCustomError>::as_ref(&self))
    }
}

impl Display for AetheriumCustomErrorWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", AsRef::<dyn AetheriumCustomError>::as_ref(&self))
    }
}

impl StdError for AetheriumCustomErrorWrapper {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl AsRef<dyn AetheriumCustomError> for AetheriumCustomErrorWrapper {
    fn as_ref(&self) -> &dyn AetheriumCustomError {
        self.0.as_ref()
    }
}

impl Deref for AetheriumCustomErrorWrapper {
    type Target = Box<dyn AetheriumCustomError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// ChainCommunicationError contains errors returned when attempting to
/// call a chain or dispatch a transaction
#[derive(Debug, thiserror::Error)]
pub enum ChainCommunicationError {
    /// Aetherium Error
    #[error(transparent)]
    AetheriumProtocolError(#[from] AetheriumProtocolError),
    /// An error with a contract call
    #[error(transparent)]
    ContractError(AetheriumCustomErrorWrapper),
    /// A transaction was dropped from the mempool
    #[error("Transaction dropped from mempool {0:?}")]
    TransactionDropped(H256),
    /// Any other error; does not implement `From` to prevent
    /// conflicting/absorbing other errors.
    #[error(transparent)]
    Other(AetheriumCustomErrorWrapper),
    /// A transaction submission timed out
    #[error("Transaction submission timed out")]
    TransactionTimeout,
    /// No signer is available and was required for the operation
    #[error("Signer unavailable")]
    SignerUnavailable,
    /// Batching transaction failed
    #[error("Batching transaction failed")]
    BatchingFailed,
    /// Cannot submit empty batch
    #[error("Cannot submit empty batch")]
    BatchIsEmpty,
    /// Failed to parse strings or integers
    #[error("Data parsing error {0:?}")]
    StrOrIntParseError(#[from] StrOrIntParseError),
    /// utf8 error
    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),
    /// Serde JSON error
    #[error("{0}")]
    JsonParseError(#[from] serde_json::Error),
    /// String hex parsing error
    #[error("{0}")]
    HexParseError(#[from] hex::FromHexError),
    /// Uint hex parsing error
    #[error("{0}")]
    UintParseError(#[from] uint::FromHexError),
    /// Decimal string parsing error
    #[error("{0}")]
    FromDecStrError(#[from] uint::FromDecStrErr),
    /// Int string parsing error
    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    /// Hash string parsing error
    #[error("{0}")]
    HashParsingError(#[from] fixed_hash::rustc_hex::FromHexError),
    /// Invalid Request
    #[error("Invalid Request: {msg:?}")]
    InvalidRequest {
        /// Error message
        msg: String,
    },
    /// Parse Error
    #[error("ParseError: {msg:?}")]
    ParseError {
        /// Error message
        msg: String,
    },
    /// Insufficient funds.
    #[error("Insufficient funds. Required: {required:?}, available: {available:?}")]
    InsufficientFunds {
        /// The required amount of funds.
        required: Box<U256>,
        /// The available amount of funds.
        available: Box<U256>,
    },
    /// Primitive type error
    #[error(transparent)]
    PrimitiveTypeError(#[from] PrimitiveTypeError),
    /// Big decimal parsing error
    #[error(transparent)]
    ParseBigDecimalError(#[from] ParseBigDecimalError),
    /// Rpc client error
    #[error(transparent)]
    RpcClientError(#[from] RpcClientError),
    /// Tokio join error
    #[cfg(feature = "async")]
    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),
    /// Custom error
    #[error("{0}")]
    CustomError(String),
    /// Eyre error
    #[error("{0}")]
    EyreError(#[from] eyre::Report),
    /// Aetherium signer error
    #[error("{0}")]
    AetheriumSignerError(#[from] AetheriumSignerError),
    /// Invalid reorg period
    #[error("Invalid reorg period: {0:?}")]
    InvalidReorgPeriod(ReorgPeriod),
}

impl ChainCommunicationError {
    /// Create a chain communication error from any other existing error
    pub fn from_other<E: AetheriumCustomError>(err: E) -> Self {
        Self::Other(AetheriumCustomErrorWrapper(Box::new(err)))
    }

    /// Create a chain communication error from any other existing error
    pub fn from_other_boxed<E: AetheriumCustomError>(err: Box<E>) -> Self {
        Self::Other(AetheriumCustomErrorWrapper(err))
    }

    /// Creates a chain communication error of the other error variant from a string slice
    pub fn from_other_str(err: &str) -> Self {
        #[derive(Debug)]
        #[repr(transparent)]
        struct StringError(String);
        impl Display for StringError {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
        impl StdError for StringError {}

        Self::from_contract_error(StringError(err.to_owned()))
    }

    /// Creates a chain communication error of the contract error variant from any other existing
    /// error
    pub fn from_contract_error<E>(err: E) -> Self
    where
        E: AetheriumCustomError,
    {
        Self::ContractError(AetheriumCustomErrorWrapper(Box::new(err)))
    }

    /// Creates a chain communication error of the contract error variant from any other existing
    /// error
    pub fn from_contract_error_boxed<E>(err: Box<E>) -> Self
    where
        E: AetheriumCustomError,
    {
        Self::ContractError(AetheriumCustomErrorWrapper(err))
    }

    /// Creates a chain communication error of the contract error variant from a static string
    pub fn from_contract_error_str(err: &'static str) -> Self {
        #[derive(Debug)]
        #[repr(transparent)]
        struct StringError(&'static str);
        impl Display for StringError {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.0)
            }
        }
        impl StdError for StringError {}

        Self::from_contract_error(StringError(err))
    }
}

impl From<AetheriumProviderError> for ChainCommunicationError {
    fn from(e: AetheriumProviderError) -> Self {
        Self::from_other(e)
    }
}

#[cfg(feature = "ethers")]
impl<T: ethers_providers::Middleware + 'static> From<ethers_contract::ContractError<T>>
    for ChainCommunicationError
{
    fn from(err: ethers_contract::ContractError<T>) -> Self {
        Self::ContractError(AetheriumCustomErrorWrapper(Box::new(err)))
    }
}

#[cfg(feature = "ethers")]
impl From<ethers_providers::ProviderError> for ChainCommunicationError {
    fn from(err: ethers_providers::ProviderError) -> Self {
        Self::ContractError(AetheriumCustomErrorWrapper(Box::new(err)))
    }
}

/// Error types for the Aetherium protocol
#[derive(Debug, thiserror::Error)]
pub enum AetheriumProtocolError {
    /// Signature Error pasthrough
    #[cfg(feature = "ethers")]
    #[error(transparent)]
    SignatureError(#[from] Box<ethers_core::types::SignatureError>),
    /// IO error from Read/Write usage
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// An unknown or invalid domain id was encountered
    #[error("Unknown or invalid domain ID ({0})")]
    UnknownDomainId(u32),
    /// Expected a gas limit and none was provided
    #[error("A gas limit was expected for `process` contract call")]
    ProcessGasLimitRequired,
}
