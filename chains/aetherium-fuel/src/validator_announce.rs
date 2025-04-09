use async_trait::async_trait;

use aetherium_core::{
    Announcement, ChainResult, AetheriumChain, AetheriumContract, AetheriumDomain,
    AetheriumProvider, SignedType, TxOutcome, ValidatorAnnounce, H256, U256,
};

/// A reference to a ValidatorAnnounce contract on some Fuel chain
#[derive(Debug)]
pub struct FuelValidatorAnnounce {}

impl AetheriumContract for FuelValidatorAnnounce {
    fn address(&self) -> H256 {
        todo!()
    }
}

impl AetheriumChain for FuelValidatorAnnounce {
    fn domain(&self) -> &AetheriumDomain {
        todo!()
    }

    fn provider(&self) -> Box<dyn AetheriumProvider> {
        todo!()
    }
}

#[async_trait]
impl ValidatorAnnounce for FuelValidatorAnnounce {
    async fn get_announced_storage_locations(
        &self,
        validators: &[H256],
    ) -> ChainResult<Vec<Vec<String>>> {
        todo!()
    }

    async fn announce(&self, announcement: SignedType<Announcement>) -> ChainResult<TxOutcome> {
        todo!()
    }

    async fn announce_tokens_needed(&self, announcement: SignedType<Announcement>) -> Option<U256> {
        todo!()
    }
}
