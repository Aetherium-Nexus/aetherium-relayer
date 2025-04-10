// TODO: re-enable clippy warnings
#![allow(dead_code)]

use std::path::PathBuf;

use aetherium_base::settings::{ChainConf, RawChainConf};
use aetherium_core::AetheriumDomain;
use tokio::task::JoinHandle;
use tracing::instrument::Instrumented;

use crate::chain_tx_adapter::{AdaptsChain, ChainTxAdapterBuilder};

/// Settings for `PayloadDispatcher`
#[derive(Debug)]
pub struct PayloadDispatcherSettings {
    // settings needed for the protocol-specific adapter
    chain_conf: ChainConf,
    /// settings needed for chain-specific adapter
    raw_chain_conf: RawChainConf,
    domain: AetheriumDomain,
    db_path: PathBuf,
}

pub struct PayloadDispatcherState {
    // db: DispatcherDb,
    adapter: Box<dyn AdaptsChain>,
}

impl PayloadDispatcherState {
    pub fn new(settings: PayloadDispatcherSettings) -> Self {
        let adapter = ChainTxAdapterBuilder::build(&settings.chain_conf, &settings.raw_chain_conf);
        Self { adapter }
    }

    pub(crate) fn from_adapter(adapter: Box<dyn AdaptsChain>) -> Self {
        Self { adapter }
    }
}
pub struct PayloadDispatcher {
    inner: PayloadDispatcherState,
}

impl PayloadDispatcher {
    pub fn new(settings: PayloadDispatcherSettings) -> Self {
        Self {
            inner: PayloadDispatcherState::new(settings),
        }
    }

    pub fn spawn(self) -> Instrumented<JoinHandle<()>> {
        // create the submit queue and channels for the Dispatcher stages
        // spawn the DbLoader with references to the submit queue and channels
        // spawn the 3 stages using the adapter, db, queue and channels
        todo!()
    }
}
