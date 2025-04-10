use std::fmt::Debug;

use async_trait::async_trait;
use auto_impl::auto_impl;

use crate::{ChainResult, AetheriumContract, AetheriumMessage, H256};

/// Interface for the MultisigIsm chain contract. Allows abstraction over
/// different chains
#[async_trait]
#[auto_impl(&, Box, Arc)]
pub trait MultisigIsm: AetheriumContract + Send + Sync + Debug {
    /// Returns the validator and threshold needed to verify message
    async fn validators_and_threshold(
        &self,
        message: &AetheriumMessage,
    ) -> ChainResult<(Vec<H256>, u8)>;
}
