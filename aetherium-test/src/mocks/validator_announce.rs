#![allow(non_snake_case)]
use core::fmt::Debug;
use mockall::*;

use async_trait::async_trait;
use aetherium_core::*;

mock! {
    pub ValidatorAnnounceContract {
        fn _domain(&self) -> &AetheriumDomain;
        fn _provider(&self) -> Box<dyn AetheriumProvider>;
        fn _address(&self) -> H256;
        fn _get_announced_storage_locations(
            &self,
            validators: &[H256],
        ) -> ChainResult<Vec<Vec<String>>>;
        fn _announce(
            &self,
            announcement: SignedType<Announcement>,
        ) -> ChainResult<TxOutcome>;
        fn _announce_tokens_needed(
            &self,
            announcement: SignedType<Announcement>,
        ) -> Option<U256>;
    }
}

impl AetheriumChain for MockValidatorAnnounceContract {
    fn domain(&self) -> &AetheriumDomain {
        self._domain()
    }

    fn provider(&self) -> Box<dyn AetheriumProvider> {
        self._provider()
    }
}

impl AetheriumContract for MockValidatorAnnounceContract {
    fn address(&self) -> H256 {
        self._address()
    }
}

impl Debug for MockValidatorAnnounceContract {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[async_trait]
impl ValidatorAnnounce for MockValidatorAnnounceContract {
    async fn get_announced_storage_locations(
        &self,
        validators: &[H256],
    ) -> ChainResult<Vec<Vec<String>>> {
        self._get_announced_storage_locations(validators)
    }

    async fn announce(&self, announcement: SignedType<Announcement>) -> ChainResult<TxOutcome> {
        self._announce(announcement)
    }
    async fn announce_tokens_needed(&self, announcement: SignedType<Announcement>) -> Option<U256> {
        self._announce_tokens_needed(announcement)
    }
}
