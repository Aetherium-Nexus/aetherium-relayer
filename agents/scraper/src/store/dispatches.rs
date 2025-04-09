use std::collections::HashMap;

use async_trait::async_trait;
use eyre::Result;

use aetherium_core::{
    unwrap_or_none_result, AetheriumLogStore, AetheriumMessage,
    AetheriumSequenceAwareIndexerStoreReader, Indexed, LogMeta, H512,
};

use crate::db::StorableMessage;
use crate::store::storage::{AetheriumDbStore, TxnWithId};

#[async_trait]
impl AetheriumLogStore<AetheriumMessage> for AetheriumDbStore {
    /// Store dispatched messages from the origin mailbox into the database.
    /// We store only messages from blocks and transaction which we could successfully insert
    /// into database.
    async fn store_logs(&self, messages: &[(Indexed<AetheriumMessage>, LogMeta)]) -> Result<u32> {
        if messages.is_empty() {
            return Ok(0);
        }
        let txns: HashMap<H512, TxnWithId> = self
            .ensure_blocks_and_txns(messages.iter().map(|r| &r.1))
            .await?
            .map(|t| (t.hash, t))
            .collect();
        let storable = messages
            .iter()
            .filter_map(|(message, meta)| {
                txns.get(&meta.transaction_id)
                    .map(|t| (message.inner().clone(), meta, t.id))
            })
            .map(|(msg, meta, txn_id)| StorableMessage { msg, meta, txn_id });
        let stored = self
            .db
            .store_dispatched_messages(self.domain.id(), &self.mailbox_address, storable)
            .await?;
        Ok(stored as u32)
    }
}

#[async_trait]
impl AetheriumSequenceAwareIndexerStoreReader<AetheriumMessage> for AetheriumDbStore {
    /// Gets a message by its nonce.
    async fn retrieve_by_sequence(&self, sequence: u32) -> Result<Option<AetheriumMessage>> {
        let message = self
            .db
            .retrieve_dispatched_message_by_nonce(self.domain.id(), &self.mailbox_address, sequence)
            .await?;
        Ok(message)
    }

    /// Gets the block number at which the log occurred.
    async fn retrieve_log_block_number_by_sequence(&self, sequence: u32) -> Result<Option<u64>> {
        let tx_id = unwrap_or_none_result!(
            self.db
                .retrieve_dispatched_tx_id(self.domain.id(), &self.mailbox_address, sequence)
                .await?
        );
        let block_id = unwrap_or_none_result!(self.db.retrieve_block_id(tx_id).await?);
        Ok(self.db.retrieve_block_number(block_id).await?)
    }
}
