#![allow(clippy::blocks_in_conditions)] // TODO: `rustc` 1.80.1 clippy issue
#![allow(clippy::unnecessary_get_then_check)] // TODO: `rustc` 1.80.1 clippy issue

use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use eyre::Result;
use aetherium_core::{AetheriumMessage, InterchainSecurityModule, ModuleType, H256};

use tracing::instrument;

use crate::msg::{
    metadata::base_builder::BuildsBaseMetadata,
    pending_message::{ISM_MAX_COUNT, ISM_MAX_DEPTH},
};

use super::{
    aggregation::AggregationIsmMetadataBuilder,
    base::{IsmWithMetadataAndType, MessageMetadataBuildParams, MetadataBuildError},
    ccip_read::CcipReadIsmMetadataBuilder,
    multisig::{MerkleRootMultisigMetadataBuilder, MessageIdMultisigMetadataBuilder},
    null_metadata::NullMetadataBuilder,
    routing::RoutingIsmMetadataBuilder,
    Metadata, MetadataBuilder,
};

/// Builds metadata for a message.
#[derive(Debug, Clone)]
pub struct MessageMetadataBuilder {
    pub base: Arc<dyn BuildsBaseMetadata>,
    pub app_context: Option<String>,
    pub max_ism_depth: u32,
    pub max_ism_count: u32,
}

/// This is the entry point for recursively building ISM metadata.
/// MessageMetadataBuilder acts as the state of the recursion.
/// Recursion works by creating additional Builders that not only impl MetadataBuilder
/// but also takes in an inner MessageMetadataBuilder when instantiated
/// to keep the recursion state.
/// ie. AggregationIsmMetadataBuilder, RoutingIsmMetadataBuilder
/// Logic-wise, it will look something like
/// MessageMetadataBuilder.build()
///   |
///   +-> RoutingIsmMetadataBuilder::new(self.clone()).build()
///         |
///         +-> self.base_builder().build()
///                    |
///                 MessageMetadataBuilder
#[async_trait]
impl MetadataBuilder for MessageMetadataBuilder {
    #[instrument(err, skip(self, message, params), fields(destination_domain=self.base_builder().destination_domain().name()))]
    async fn build(
        &self,
        ism_address: H256,
        message: &AetheriumMessage,
        params: MessageMetadataBuildParams,
    ) -> Result<Metadata, MetadataBuildError> {
        build_message_metadata(self.clone(), ism_address, message, params)
            .await
            .map(|res| res.metadata)
    }
}

impl MessageMetadataBuilder {
    pub async fn new(
        base: Arc<dyn BuildsBaseMetadata>,
        ism_address: H256,
        message: &AetheriumMessage,
    ) -> Result<Self> {
        let app_context = base
            .app_context_classifier()
            .get_app_context(message, ism_address)
            .await?;
        Ok(Self {
            base,
            app_context,
            max_ism_depth: ISM_MAX_DEPTH,
            max_ism_count: ISM_MAX_COUNT,
        })
    }

    pub fn base_builder(&self) -> &Arc<dyn BuildsBaseMetadata> {
        &self.base
    }
}

/// Builds metadata for a message.
pub async fn build_message_metadata(
    message_builder: MessageMetadataBuilder,
    ism_address: H256,
    message: &AetheriumMessage,
    mut params: MessageMetadataBuildParams,
) -> Result<IsmWithMetadataAndType, MetadataBuildError> {
    let ism: Box<dyn InterchainSecurityModule> = message_builder
        .base_builder()
        .build_ism(ism_address)
        .await
        .map_err(|err| MetadataBuildError::FailedToBuild(err.to_string()))?;

    let module_type = ism
        .module_type()
        .await
        .map_err(|err| MetadataBuildError::FailedToBuild(err.to_string()))?;

    // check if max depth is reached
    if params.ism_depth >= message_builder.max_ism_depth {
        tracing::error!(
            ism_depth = message_builder.max_ism_depth,
            ism_address = ?ism_address,
            message_id = ?message.id(),
            "Max ISM depth reached",
        );
        return Err(MetadataBuildError::MaxIsmDepthExceeded(
            message_builder.max_ism_depth,
        ));
    }
    params.ism_depth = params.ism_depth.saturating_add(1);
    {
        // check if max ism count is reached
        let mut ism_count = params.ism_count.lock().await;
        if *ism_count >= message_builder.max_ism_count {
            tracing::error!(
                ism_count = message_builder.max_ism_count,
                ism_address = ?ism_address,
                message_id = ?message.id(),
                "Max ISM count reached",
            );
            return Err(MetadataBuildError::MaxIsmCountReached(
                message_builder.max_ism_count,
            ));
        }
        *ism_count = ism_count.saturating_add(1);
    }

    let metadata_builder: Box<dyn MetadataBuilder> = match module_type {
        ModuleType::MerkleRootMultisig => {
            Box::new(MerkleRootMultisigMetadataBuilder::new(message_builder))
        }
        ModuleType::MessageIdMultisig => {
            Box::new(MessageIdMultisigMetadataBuilder::new(message_builder))
        }
        ModuleType::Routing => Box::new(RoutingIsmMetadataBuilder::new(message_builder)),
        ModuleType::Aggregation => Box::new(AggregationIsmMetadataBuilder::new(message_builder)),
        ModuleType::Null => Box::new(NullMetadataBuilder::new()),
        ModuleType::CcipRead => Box::new(CcipReadIsmMetadataBuilder::new(message_builder)),
        _ => return Err(MetadataBuildError::UnsupportedModuleType(module_type)),
    };
    let metadata = metadata_builder.build(ism_address, message, params).await?;

    Ok(IsmWithMetadataAndType { ism, metadata })
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use aetherium_core::{
        AetheriumDomain, AetheriumMessage, KnownAetheriumDomain, Mailbox, ModuleType, H256, U256,
    };
    use aetherium_test::mocks::MockMailboxContract;

    use crate::{
        msg::metadata::{
            base::MetadataBuildError, message_builder::build_message_metadata,
            IsmAwareAppContextClassifier, MessageMetadataBuildParams,
        },
        settings::matching_list::{Filter, ListElement, MatchingList},
        test_utils::{
            mock_aggregation_ism::MockAggregationIsm, mock_base_builder::MockBaseMetadataBuilder,
            mock_ism::MockInterchainSecurityModule, mock_routing_ism::MockRoutingIsm,
        },
    };

    use super::MessageMetadataBuilder;

    fn build_mock_base_builder() -> MockBaseMetadataBuilder {
        let origin_domain = AetheriumDomain::Known(KnownAetheriumDomain::Optimism);
        let destination_domain = AetheriumDomain::Known(KnownAetheriumDomain::Ethereum);

        let mut base_builder = MockBaseMetadataBuilder::new();
        base_builder.responses.origin_domain = Some(origin_domain.clone());
        base_builder.responses.destination_domain = Some(destination_domain);

        let mock_mailbox = MockMailboxContract::new();
        let mailbox: Arc<dyn Mailbox> = Arc::new(mock_mailbox);
        let app_context_classifier = IsmAwareAppContextClassifier::new(
            mailbox,
            vec![(
                MatchingList(Some(vec![ListElement::new(
                    Filter::Wildcard,
                    Filter::Wildcard,
                    Filter::Wildcard,
                    Filter::Wildcard,
                    Filter::Wildcard,
                )])),
                "abcd".to_string(),
            )],
        );
        base_builder.responses.app_context_classifier = Some(app_context_classifier);
        base_builder
    }

    fn insert_null_isms(base_builder: &MockBaseMetadataBuilder, addresses: &[H256]) {
        for ism_address in addresses {
            let mock_ism = MockInterchainSecurityModule::new(*ism_address);
            mock_ism
                .responses
                .module_type
                .lock()
                .unwrap()
                .push_back(Ok(ModuleType::Null));
            mock_ism
                .responses
                .dry_run_verify
                .lock()
                .unwrap()
                .push_back(Ok(Some(U256::zero())));
            base_builder
                .responses
                .push_build_ism_response(*ism_address, Ok(Box::new(mock_ism)));
        }
    }

    fn insert_mock_routing_isms(
        base_builder: &MockBaseMetadataBuilder,
        addresses: &[(H256, H256)],
    ) {
        for (ism_address, route_address) in addresses {
            let mock_ism = MockInterchainSecurityModule::new(*ism_address);
            mock_ism
                .responses
                .module_type
                .lock()
                .unwrap()
                .push_back(Ok(ModuleType::Routing));
            mock_ism
                .responses
                .dry_run_verify
                .lock()
                .unwrap()
                .push_back(Ok(Some(U256::zero())));
            base_builder
                .responses
                .push_build_ism_response(*ism_address, Ok(Box::new(mock_ism)));

            let routing_ism = MockRoutingIsm::default();
            routing_ism
                .responses
                .route
                .lock()
                .unwrap()
                .push_back(Ok(*route_address));
            base_builder
                .responses
                .build_routing_ism
                .lock()
                .unwrap()
                .push_back(Ok(Box::new(routing_ism)));
        }
    }

    fn insert_mock_aggregation_isms(
        base_builder: &MockBaseMetadataBuilder,
        addresses: Vec<(H256, Vec<H256>, u8)>,
    ) {
        for (ism_address, aggregation_addresses, threshold) in addresses {
            let mock_ism = MockInterchainSecurityModule::new(ism_address);
            mock_ism
                .responses
                .module_type
                .lock()
                .unwrap()
                .push_back(Ok(ModuleType::Aggregation));
            mock_ism
                .responses
                .dry_run_verify
                .lock()
                .unwrap()
                .push_back(Ok(Some(U256::zero())));
            base_builder
                .responses
                .push_build_ism_response(ism_address, Ok(Box::new(mock_ism)));

            let agg_ism = MockAggregationIsm::default();
            agg_ism
                .responses
                .modules_and_threshold
                .lock()
                .unwrap()
                .push_back(Ok((aggregation_addresses, threshold)));
            base_builder
                .responses
                .build_aggregation_ism
                .lock()
                .unwrap()
                .push_back(Ok(Box::new(agg_ism)));
        }
    }

    /// 0x0
    ///  |
    ///  +---> 0x100
    ///  |       |
    ///  |       +----> 0x110 -> 0x1100
    ///  |       |
    ///  |       +----> 0x120 -> 0x1200
    ///  |
    ///  +---> 0x200
    ///  |       |
    ///  |       +----> 0x210 -> 0x2100
    ///  |       |
    ///  |       +----> 0x220 -> 0x2200
    ///  |
    ///  +---> 0x300
    ///          |
    ///          +----> 0x310 -> 0x3100
    ///          |
    ///          +----> 0x320 -> 0x3200
    fn insert_ism_test_data(base_builder: &MockBaseMetadataBuilder) {
        insert_mock_aggregation_isms(
            base_builder,
            vec![
                (
                    H256::from_low_u64_be(0),
                    vec![
                        H256::from_low_u64_be(100),
                        H256::from_low_u64_be(200),
                        H256::from_low_u64_be(300),
                    ],
                    2,
                ),
                (
                    H256::from_low_u64_be(100),
                    vec![H256::from_low_u64_be(110), H256::from_low_u64_be(120)],
                    2,
                ),
                (
                    H256::from_low_u64_be(200),
                    vec![H256::from_low_u64_be(210), H256::from_low_u64_be(220)],
                    2,
                ),
                (
                    H256::from_low_u64_be(300),
                    vec![H256::from_low_u64_be(310), H256::from_low_u64_be(320)],
                    2,
                ),
            ],
        );

        insert_mock_routing_isms(
            base_builder,
            &[
                (H256::from_low_u64_be(110), H256::from_low_u64_be(1100)),
                (H256::from_low_u64_be(120), H256::from_low_u64_be(1200)),
                (H256::from_low_u64_be(210), H256::from_low_u64_be(2100)),
                (H256::from_low_u64_be(220), H256::from_low_u64_be(2200)),
                (H256::from_low_u64_be(310), H256::from_low_u64_be(3100)),
                (H256::from_low_u64_be(320), H256::from_low_u64_be(3200)),
            ],
        );

        insert_null_isms(
            base_builder,
            &[
                H256::from_low_u64_be(1100),
                H256::from_low_u64_be(1200),
                H256::from_low_u64_be(2100),
                H256::from_low_u64_be(2200),
                H256::from_low_u64_be(3100),
                H256::from_low_u64_be(3200),
            ],
        );
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn depth_already_reached() {
        let base_builder = build_mock_base_builder();
        insert_null_isms(&base_builder, &[H256::zero()]);

        let ism_address = H256::zero();
        let message = AetheriumMessage::default();
        let message_builder = {
            let mut builder =
                MessageMetadataBuilder::new(Arc::new(base_builder), ism_address, &message)
                    .await
                    .expect("Failed to build MessageMetadataBuilder");
            builder.max_ism_depth = 0;
            builder
        };
        let params = MessageMetadataBuildParams::default();
        let err = build_message_metadata(message_builder, ism_address, &message, params.clone())
            .await
            .err()
            .expect("Metadata found when it should have failed");
        assert_eq!(err, MetadataBuildError::MaxIsmDepthExceeded(0));
        assert_eq!(*(params.ism_count.lock().await), 0);

        assert!(logs_contain("Max ISM depth reached ism_depth=0"));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn ism_count_already_reached() {
        let base_builder = build_mock_base_builder();
        insert_null_isms(&base_builder, &[H256::zero()]);

        let ism_address = H256::zero();
        let message = AetheriumMessage::default();

        let message_builder = {
            let mut builder =
                MessageMetadataBuilder::new(Arc::new(base_builder), ism_address, &message)
                    .await
                    .expect("Failed to build MessageMetadataBuilder");
            builder.max_ism_count = 0;
            builder
        };

        let params = MessageMetadataBuildParams::default();
        let err = build_message_metadata(message_builder, ism_address, &message, params.clone())
            .await
            .err()
            .expect("Metadata found when it should have failed");
        assert_eq!(err, MetadataBuildError::MaxIsmCountReached(0));
        assert_eq!(*(params.ism_count.lock().await), 0);

        assert!(logs_contain("Max ISM count reached ism_count=0"));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn max_depth_reached() {
        let base_builder = build_mock_base_builder();
        insert_ism_test_data(&base_builder);

        let ism_address = H256::zero();
        let message = AetheriumMessage::default();

        let message_builder = {
            let mut builder =
                MessageMetadataBuilder::new(Arc::new(base_builder), ism_address, &message)
                    .await
                    .expect("Failed to build MessageMetadataBuilder");
            builder.max_ism_depth = 2;
            builder
        };

        let params = MessageMetadataBuildParams::default();
        let err = build_message_metadata(message_builder, ism_address, &message, params.clone())
            .await
            .err()
            .expect("Metadata found when it should have failed");
        assert_eq!(err, MetadataBuildError::AggregationThresholdNotMet(2));
        assert!(*(params.ism_count.lock().await) <= 4);
        assert!(logs_contain("Max ISM depth reached ism_depth=2"));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn max_ism_count_reached() {
        let base_builder = build_mock_base_builder();
        insert_ism_test_data(&base_builder);

        let ism_address = H256::zero();
        let message = AetheriumMessage::default();

        let message_builder = {
            let mut builder =
                MessageMetadataBuilder::new(Arc::new(base_builder), ism_address, &message)
                    .await
                    .expect("Failed to build MessageMetadataBuilder");
            builder.max_ism_count = 5;
            builder
        };

        let params = MessageMetadataBuildParams::default();
        let err = build_message_metadata(message_builder, ism_address, &message, params.clone())
            .await
            .err()
            .expect("Metadata found when it should have failed");
        assert_eq!(err, MetadataBuildError::AggregationThresholdNotMet(2));
        assert_eq!(*(params.ism_count.lock().await), 5);
        assert!(logs_contain("Max ISM count reached ism_count=5"));
    }
}
