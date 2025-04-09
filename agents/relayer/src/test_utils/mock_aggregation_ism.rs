use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use aetherium_core::{
    AggregationIsm, ChainResult, AetheriumChain, AetheriumContract, AetheriumDomain,
    AetheriumMessage, H256,
};

type ResponseList<T> = Arc<Mutex<VecDeque<T>>>;

#[derive(Debug, Default)]
pub struct MockAggregationIsmResponses {
    pub modules_and_threshold: ResponseList<ChainResult<(Vec<H256>, u8)>>,
    pub domain: Option<AetheriumDomain>,
}

#[derive(Debug, Default)]
pub struct MockAggregationIsm {
    pub responses: MockAggregationIsmResponses,
}

#[async_trait::async_trait]
impl AggregationIsm for MockAggregationIsm {
    async fn modules_and_threshold(
        &self,
        _message: &AetheriumMessage,
    ) -> ChainResult<(Vec<H256>, u8)> {
        self.responses
            .modules_and_threshold
            .lock()
            .unwrap()
            .pop_front()
            .expect("No mock modules_and_threshold response set")
    }
}

impl AetheriumContract for MockAggregationIsm {
    fn address(&self) -> H256 {
        H256::zero()
    }
}

impl AetheriumChain for MockAggregationIsm {
    fn domain(&self) -> &aetherium_core::AetheriumDomain {
        self.responses
            .domain
            .as_ref()
            .expect("No mock domain response set")
    }
    fn provider(&self) -> Box<dyn aetherium_core::AetheriumProvider> {
        unimplemented!()
    }
}
