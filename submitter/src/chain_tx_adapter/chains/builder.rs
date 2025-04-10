// TODO: re-enable clippy warnings
#![allow(dead_code)]

use aetherium_base::settings::{ChainConf, RawChainConf};
use aetherium_core::{AetheriumDomain, AetheriumDomainProtocol};

use crate::chain_tx_adapter::{
    chains::{cosmos::CosmosTxAdapter, ethereum::EthereumTxAdapter, sealevel::SealevelTxAdapter},
    AdaptsChain,
};

pub struct ChainTxAdapterBuilder {}

impl ChainTxAdapterBuilder {
    pub fn build(conf: &ChainConf, raw_conf: &RawChainConf) -> Box<dyn AdaptsChain> {
        use AetheriumDomainProtocol::*;

        let adapter: Box<dyn AdaptsChain> = match conf.domain.domain_protocol() {
            Ethereum => Box::new(EthereumTxAdapter::new(conf.clone(), raw_conf.clone())),
            Fuel => todo!(),
            Sealevel => Box::new(SealevelTxAdapter::new(conf.clone(), raw_conf.clone())),
            Cosmos => Box::new(CosmosTxAdapter::new(conf.clone(), raw_conf.clone())),
        };

        adapter
    }
}
