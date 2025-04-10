use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use derive_new::new;
use ethers::prelude::Middleware;
use ethers_core::{abi::Address, types::BlockNumber};
use aetherium_core::{ethers_core_types, ChainInfo, AetheriumCustomErrorWrapper, H512, U256};
use tokio::time::sleep;
use tracing::instrument;

use aetherium_core::{
    BlockInfo, ChainCommunicationError, ChainResult, ContractLocator, AetheriumChain,
    AetheriumDomain, AetheriumProvider, AetheriumProviderError, TxnInfo, TxnReceiptInfo, H256,
};

use crate::{BuildableWithProvider, ConnectionConf};

/// Connection to an ethereum provider. Useful for querying information about
/// the blockchain.
#[derive(Debug, Clone, new)]
pub struct EthereumProvider<M> {
    provider: Arc<M>,
    domain: AetheriumDomain,
}

impl<M> AetheriumChain for EthereumProvider<M>
where
    M: Middleware + 'static,
{
    fn domain(&self) -> &AetheriumDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn AetheriumProvider> {
        Box::new(EthereumProvider::new(
            self.provider.clone(),
            self.domain.clone(),
        ))
    }
}

#[async_trait]
impl<M> AetheriumProvider for EthereumProvider<M>
where
    M: Middleware + 'static,
{
    #[instrument(err, skip(self))]
    #[allow(clippy::blocks_in_conditions)] // TODO: `rustc` 1.80.1 clippy issue
    async fn get_block_by_height(&self, height: u64) -> ChainResult<BlockInfo> {
        let block = get_with_retry_on_none(
            &height,
            |h| self.provider.get_block(*h),
            |h| AetheriumProviderError::CouldNotFindBlockByHeight(*h),
        )
        .await?;

        let block_height = block
            .number
            .ok_or(AetheriumProviderError::CouldNotFindBlockByHeight(height))?
            .as_u64();

        if block_height != height {
            Err(AetheriumProviderError::IncorrectBlockByHeight(
                height,
                block_height,
            ))?;
        }

        let block_hash = block
            .hash
            .ok_or(AetheriumProviderError::BlockWithoutHash(height))?;

        let block_info = BlockInfo {
            hash: block_hash.into(),
            timestamp: block.timestamp.as_u64(),
            number: block_height,
        };

        Ok(block_info)
    }

    #[instrument(err, skip(self))]
    #[allow(clippy::blocks_in_conditions)] // TODO: `rustc` 1.80.1 clippy issue
    async fn get_txn_by_hash(&self, hash: &H512) -> ChainResult<TxnInfo> {
        let txn = get_with_retry_on_none(
            hash,
            |h| self.provider.get_transaction(*h),
            |h| AetheriumProviderError::CouldNotFindTransactionByHash(*h),
        )
        .await?;

        let receipt = self
            .provider
            .get_transaction_receipt(*hash)
            .await
            .map_err(ChainCommunicationError::from_other)?
            .map(|r| -> Result<_, AetheriumProviderError> {
                Ok(TxnReceiptInfo {
                    gas_used: r.gas_used.ok_or(AetheriumProviderError::NoGasUsed)?.into(),
                    cumulative_gas_used: r.cumulative_gas_used.into(),
                    effective_gas_price: r.effective_gas_price.map(Into::into),
                })
            })
            .transpose()?;

        let txn_info = TxnInfo {
            hash: *hash,
            max_fee_per_gas: txn.max_fee_per_gas.map(Into::into),
            max_priority_fee_per_gas: txn.max_priority_fee_per_gas.map(Into::into),
            gas_price: txn.gas_price.map(Into::into),
            gas_limit: txn.gas.into(),
            nonce: txn.nonce.as_u64(),
            sender: txn.from.into(),
            recipient: txn.to.map(Into::into),
            receipt,
            raw_input_data: Some(txn.input.to_vec()),
        };

        Ok(txn_info)
    }

    #[instrument(err, skip(self))]
    #[allow(clippy::blocks_in_conditions)] // TODO: `rustc` 1.80.1 clippy issue
    async fn is_contract(&self, address: &H256) -> ChainResult<bool> {
        let code = self
            .provider
            .get_code(ethers_core_types::H160::from(*address), None)
            .await
            .map_err(ChainCommunicationError::from_other)?;
        Ok(!code.is_empty())
    }

    #[instrument(err, skip(self))]
    #[allow(clippy::blocks_in_conditions)] // TODO: `rustc` 1.80.1 clippy issue
    async fn get_balance(&self, address: String) -> ChainResult<U256> {
        // Can't use the address directly as a string, because ethers interprets it
        // as an ENS name rather than an address.
        let addr: Address = address.parse()?;
        let balance = self
            .provider
            .get_balance(addr, None)
            .await
            .map_err(ChainCommunicationError::from_other)?;
        Ok(balance.into())
    }

    async fn get_chain_metrics(&self) -> ChainResult<Option<ChainInfo>> {
        let Some(block) = self
            .provider
            .get_block(BlockNumber::Latest)
            .await
            .map_err(|e| {
                ChainCommunicationError::Other(AetheriumCustomErrorWrapper::new(Box::new(e)))
            })?
        else {
            tracing::trace!(domain=?self.domain, "Latest block not found");
            return Ok(None);
        };

        // Given the block is queried with `BlockNumber::Latest` rather than `BlockNumber::Pending`,
        // if `block` is Some at this point, we're guaranteed to have its `hash` and `number` defined,
        // so it's safe to unwrap below
        // more info at <https://docs.rs/ethers/latest/ethers/core/types/struct.Block.html#structfield.number>
        let chain_metrics = ChainInfo::new(
            BlockInfo {
                hash: block.hash.unwrap().into(),
                timestamp: block.timestamp.as_u64(),
                number: block.number.unwrap().as_u64(),
            },
            block.base_fee_per_gas.map(Into::into),
        );
        Ok(Some(chain_metrics))
    }
}

impl<M> EthereumProvider<M>
where
    M: Middleware + 'static,
{
    #[instrument(err, skip(self))]
    async fn get_storage_at(&self, address: H256, location: H256) -> ChainResult<H256> {
        let storage = self
            .provider
            .get_storage_at(
                ethers_core_types::H160::from(address),
                location.into(),
                None,
            )
            .await
            .map_err(ChainCommunicationError::from_other)?;
        Ok(storage.into())
    }
}

/// Builder for aetherium providers.
pub struct AetheriumProviderBuilder {}

#[async_trait]
impl BuildableWithProvider for AetheriumProviderBuilder {
    type Output = Box<dyn AetheriumProvider>;
    const NEEDS_SIGNER: bool = false;

    async fn build_with_provider<M: Middleware + 'static>(
        &self,
        provider: M,
        _conn: &ConnectionConf,
        locator: &ContractLocator,
    ) -> Self::Output {
        Box::new(EthereumProvider::new(
            Arc::new(provider),
            locator.domain.clone(),
        ))
    }
}

/// Call a get function that returns a Result<Option<T>> and retry if the inner
/// option is None. This can happen because the provider has not discovered the
/// object we are looking for yet.
async fn get_with_retry_on_none<T, F, O, E, I, N>(
    id: &I,
    get: F,
    not_found_error: N,
) -> ChainResult<T>
where
    F: Fn(&I) -> O,
    O: Future<Output = Result<Option<T>, E>>,
    E: std::error::Error + Send + Sync + 'static,
    N: Fn(&I) -> AetheriumProviderError,
{
    for _ in 0..3 {
        if let Some(t) = get(id).await.map_err(ChainCommunicationError::from_other)? {
            return Ok(t);
        } else {
            sleep(Duration::from_secs(5)).await;
            continue;
        };
    }
    Err(not_found_error(id).into())
}
