use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use aetherium_core::{
    ChainResult, AetheriumChain, AetheriumContract, AetheriumDomain, AetheriumMessage,
    InterchainSecurityModule, ModuleType, H256, U256,
};

type ResponseList<T> = Arc<Mutex<VecDeque<T>>>;

#[derive(Debug, Default)]
pub struct MockInterchainSecurityModuleResponses {
    pub module_type: ResponseList<ChainResult<ModuleType>>,
    pub dry_run_verify: ResponseList<ChainResult<Option<U256>>>,
    pub domain: Option<AetheriumDomain>,
}

pub struct MockInterchainSecurityModule {
    pub responses: MockInterchainSecurityModuleResponses,
    pub address: H256,
}

impl std::fmt::Debug for MockInterchainSecurityModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MockInterchainSecurityModule {{ address: {} }}",
            self.address
        )
    }
}

impl MockInterchainSecurityModule {
    pub fn new(address: H256) -> Self {
        Self {
            responses: MockInterchainSecurityModuleResponses::default(),
            address,
        }
    }
}

#[async_trait::async_trait]
impl InterchainSecurityModule for MockInterchainSecurityModule {
    async fn module_type(&self) -> ChainResult<ModuleType> {
        self.responses
            .module_type
            .lock()
            .unwrap()
            .pop_front()
            .expect("No mock module_type response set")
    }

    /// Dry runs the `verify()` ISM call and returns `Some(gas_estimate)` if the call
    /// succeeds.
    async fn dry_run_verify(
        &self,
        _message: &AetheriumMessage,
        _metadata: &[u8],
    ) -> ChainResult<Option<U256>> {
        self.responses
            .dry_run_verify
            .lock()
            .unwrap()
            .pop_front()
            .expect(&format!(
                "No mock dry_run_verify response set {}",
                self.address
            ))
    }
}

impl AetheriumContract for MockInterchainSecurityModule {
    fn address(&self) -> H256 {
        H256::zero()
    }
}

impl AetheriumChain for MockInterchainSecurityModule {
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
    use crate::test_utils::mock_ism::MockInterchainSecurityModule;

    use super::*;

    /// Just to test mock structs work
    #[tokio::test]
    async fn test_mock_works() {
        let mock_ism = MockInterchainSecurityModule::new(H256::zero());
        mock_ism
            .responses
            .module_type
            .lock()
            .unwrap()
            .push_back(Ok(ModuleType::Routing));
        mock_ism
            .responses
            .module_type
            .lock()
            .unwrap()
            .push_back(Ok(ModuleType::Aggregation));

        let module_type = mock_ism.module_type().await.expect("No response");
        assert_eq!(module_type, ModuleType::Routing);

        let module_type = mock_ism.module_type().await.expect("No response");
        assert_eq!(module_type, ModuleType::Aggregation);
    }
}
