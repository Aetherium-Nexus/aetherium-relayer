use std::ops::RangeInclusive;

use async_trait::async_trait;

use aetherium_core::{
    ChainResult, AetheriumChain, AetheriumContract, Indexed, Indexer, InterchainGasPaymaster,
};
use aetherium_core::{AetheriumDomain, AetheriumProvider, InterchainGasPayment, LogMeta, H256};

/// A reference to an IGP contract on some Fuel chain
#[derive(Debug)]
pub struct FuelInterchainGasPaymaster {}

impl AetheriumContract for FuelInterchainGasPaymaster {
    fn address(&self) -> H256 {
        todo!()
    }
}

impl AetheriumChain for FuelInterchainGasPaymaster {
    fn domain(&self) -> &AetheriumDomain {
        todo!()
    }

    fn provider(&self) -> Box<dyn AetheriumProvider> {
        todo!()
    }
}

impl InterchainGasPaymaster for FuelInterchainGasPaymaster {}

/// Struct that retrieves event data for a Fuel IGP contract
#[derive(Debug)]
pub struct FuelInterchainGasPaymasterIndexer {}

#[async_trait]
impl Indexer<InterchainGasPayment> for FuelInterchainGasPaymasterIndexer {
    async fn fetch_logs_in_range(
        &self,
        range: RangeInclusive<u32>,
    ) -> ChainResult<Vec<(Indexed<InterchainGasPayment>, LogMeta)>> {
        todo!()
    }

    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        todo!()
    }
}
