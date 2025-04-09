use async_trait::async_trait;

use aetherium_core::{
    ChainResult, AetheriumChain, AetheriumContract, AetheriumDomain, AetheriumMessage,
    AetheriumProvider, RoutingIsm, H256,
};

/// A reference to a RoutingIsm contract on some Fuel chain
#[derive(Debug)]
pub struct FuelRoutingIsm {}

impl AetheriumContract for FuelRoutingIsm {
    fn address(&self) -> H256 {
        todo!()
    }
}

impl AetheriumChain for FuelRoutingIsm {
    fn domain(&self) -> &AetheriumDomain {
        todo!()
    }

    fn provider(&self) -> Box<dyn AetheriumProvider> {
        todo!()
    }
}

#[async_trait]
impl RoutingIsm for FuelRoutingIsm {
    /// Returns the ism needed to verify message
    async fn route(&self, message: &AetheriumMessage) -> ChainResult<H256> {
        todo!()
    }
}
