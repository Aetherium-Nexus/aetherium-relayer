use std::fmt::Debug;

use async_trait::async_trait;
use auto_impl::auto_impl;

use crate::{ChainResult, AetheriumContract, AetheriumMessage, H256};

/// Interface for the RoutingIsm chain contract. Allows abstraction over
/// different chains
#[async_trait]
#[auto_impl(&, Box, Arc)]
pub trait RoutingIsm: AetheriumContract + Send + Sync + Debug {
    /// Returns the ISM needed to verify message
    async fn route(&self, message: &AetheriumMessage) -> ChainResult<H256>;
}
