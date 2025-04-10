use std::fmt::Debug;

use async_trait::async_trait;
use auto_impl::auto_impl;

use crate::AetheriumContract;

/// Interface for the InterchainGasPaymaster chain contract.
/// Allows abstraction over different chains.
#[async_trait]
#[auto_impl(&, Box, Arc)]
pub trait InterchainGasPaymaster: AetheriumContract + Send + Sync + Debug {}
