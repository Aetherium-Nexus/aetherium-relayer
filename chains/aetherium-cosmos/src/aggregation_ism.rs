use std::str::FromStr;

use crate::{
    grpc::WasmProvider,
    payloads::{
        ism_routes::QueryIsmGeneralRequest,
        multisig_ism::{VerifyInfoRequest, VerifyInfoRequestInner, VerifyInfoResponse},
    },
    ConnectionConf, CosmosProvider, Signer,
};
use async_trait::async_trait;
use aetherium_core::{
    AggregationIsm, ChainResult, ContractLocator, AetheriumChain, AetheriumContract,
    AetheriumDomain, AetheriumMessage, AetheriumProvider, RawAetheriumMessage, H160, H256,
};
use tracing::instrument;

/// A reference to an AggregationIsm contract on some Cosmos chain
#[derive(Debug)]
pub struct CosmosAggregationIsm {
    domain: AetheriumDomain,
    address: H256,
    provider: Box<CosmosProvider>,
}

impl CosmosAggregationIsm {
    /// create new Cosmos AggregationIsm agent
    pub fn new(provider: CosmosProvider, locator: ContractLocator) -> ChainResult<Self> {
        Ok(Self {
            domain: locator.domain.clone(),
            address: locator.address,
            provider: Box::new(provider),
        })
    }
}

impl AetheriumContract for CosmosAggregationIsm {
    fn address(&self) -> H256 {
        self.address
    }
}

impl AetheriumChain for CosmosAggregationIsm {
    fn domain(&self) -> &AetheriumDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn AetheriumProvider> {
        self.provider.clone()
    }
}

#[async_trait]
impl AggregationIsm for CosmosAggregationIsm {
    #[allow(clippy::blocks_in_conditions)] // TODO: `rustc` 1.80.1 clippy issue
    #[instrument(err)]
    async fn modules_and_threshold(
        &self,
        message: &AetheriumMessage,
    ) -> ChainResult<(Vec<H256>, u8)> {
        let payload = VerifyInfoRequest {
            verify_info: VerifyInfoRequestInner {
                message: hex::encode(RawAetheriumMessage::from(message)),
            },
        };

        let data = self
            .provider
            .grpc()
            .wasm_query(QueryIsmGeneralRequest { ism: payload }, None)
            .await?;
        let response: VerifyInfoResponse = serde_json::from_slice(&data)?;

        // Note that due to a misnomer in the CosmWasm implementation, the `modules` field is called `validators`.
        let modules: ChainResult<Vec<H256>> = response
            .validators
            .iter()
            .map(|module| {
                // The returned values are Bech32-decoded Cosmos addresses.
                // Since they are not EOAs but rather contracts, they can be 32 bytes long and
                // need to be parsed directly as an `H256`.
                if let Ok(res) = H256::from_str(module) {
                    return Ok(res);
                }
                // If the address is not 32 bytes long, it is a 20-byte address
                H160::from_str(module).map(H256::from).map_err(Into::into)
            })
            .collect();

        Ok((modules?, response.threshold))
    }
}
