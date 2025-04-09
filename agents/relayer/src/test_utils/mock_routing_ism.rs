use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use aetherium_core::{
    ChainResult, AetheriumChain, AetheriumContract, AetheriumDomain, AetheriumMessage, RoutingIsm,
    H256,
};

type ResponseList<T> = Arc<Mutex<VecDeque<T>>>;

#[derive(Debug, Default)]
pub struct MockRoutingIsmResponses {
    pub route: ResponseList<ChainResult<H256>>,
    pub domain: Option<AetheriumDomain>,
}

#[derive(Debug, Default)]
pub struct MockRoutingIsm {
    pub responses: MockRoutingIsmResponses,
}

#[async_trait::async_trait]
impl RoutingIsm for MockRoutingIsm {
    async fn route(&self, _message: &AetheriumMessage) -> ChainResult<H256> {
        self.responses
            .route
            .lock()
            .unwrap()
            .pop_front()
            .expect("No mock route response set")
    }
}

impl AetheriumContract for MockRoutingIsm {
    fn address(&self) -> H256 {
        H256::zero()
    }
}

impl AetheriumChain for MockRoutingIsm {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Just to test mock structs work
    #[tokio::test]
    async fn test_mock_works() {
        let mock_ism = MockRoutingIsm::default();
        mock_ism
            .responses
            .route
            .lock()
            .unwrap()
            .push_back(Ok(H256::zero()));
        mock_ism
            .responses
            .route
            .lock()
            .unwrap()
            .push_back(Ok(H256::from_low_u64_le(10)));

        let message = AetheriumMessage::default();
        let module_type = mock_ism.route(&message).await.expect("No response");
        assert_eq!(module_type, H256::zero());

        let module_type = mock_ism.route(&message).await.expect("No response");
        assert_eq!(module_type, H256::from_low_u64_le(10));
    }
}
