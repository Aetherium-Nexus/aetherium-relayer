use std::str::FromStr;

use crate::{
    grpc::WasmProvider, payloads::ism_routes::QueryIsmGeneralRequest, signers::Signer,
    ConnectionConf, CosmosProvider,
};
use async_trait::async_trait;
use aetherium_core::{
    ChainResult, ContractLocator, AetheriumChain, AetheriumContract, AetheriumDomain,
    AetheriumMessage, AetheriumProvider, MultisigIsm, RawAetheriumMessage, H160, H256,
};

use crate::payloads::multisig_ism::{self, VerifyInfoRequest, VerifyInfoRequestInner};

/// A reference to a MultisigIsm contract on some Cosmos chain
#[derive(Debug)]
pub struct CosmosMultisigIsm {
    domain: AetheriumDomain,
    address: H256,
    provider: CosmosProvider,
}

impl CosmosMultisigIsm {
    /// create a new instance of CosmosMultisigIsm
    pub fn new(provider: CosmosProvider, locator: ContractLocator) -> ChainResult<Self> {
        Ok(Self {
            domain: locator.domain.clone(),
            address: locator.address,
            provider,
        })
    }
}

impl AetheriumContract for CosmosMultisigIsm {
    fn address(&self) -> H256 {
        self.address
    }
}

impl AetheriumChain for CosmosMultisigIsm {
    fn domain(&self) -> &AetheriumDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn AetheriumProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl MultisigIsm for CosmosMultisigIsm {
    /// Returns the validator and threshold needed to verify message
    async fn validators_and_threshold(
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
        let response: multisig_ism::VerifyInfoResponse = serde_json::from_slice(&data)?;

        let validators: ChainResult<Vec<H256>> = response
            .validators
            .iter()
            .map(|v| H160::from_str(v).map(H256::from).map_err(Into::into))
            .collect();

        Ok((validators?, response.threshold))
    }
}
