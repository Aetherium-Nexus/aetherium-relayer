use async_trait::async_trait;

use aetherium_core::{
    ChainResult, AetheriumChain, AetheriumContract, AetheriumDomain, AetheriumMessage,
    AetheriumProvider, MultisigIsm, H256,
};

/// A reference to a MultisigIsm contract on some Fuel chain
#[derive(Debug)]
pub struct FuelMultisigIsm {}

impl AetheriumContract for FuelMultisigIsm {
    fn address(&self) -> H256 {
        todo!()
    }
}

impl AetheriumChain for FuelMultisigIsm {
    fn domain(&self) -> &AetheriumDomain {
        todo!()
    }

    fn provider(&self) -> Box<dyn AetheriumProvider> {
        todo!()
    }
}

#[async_trait]
impl MultisigIsm for FuelMultisigIsm {
    /// Returns the validator and threshold needed to verify message
    async fn validators_and_threshold(
        &self,
        message: &AetheriumMessage,
    ) -> ChainResult<(Vec<H256>, u8)> {
        todo!()
    }
}
