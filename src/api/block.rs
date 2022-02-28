use crate::api::error::{ApiError, ApiResult};
use crate::api::query::{block_hash_from_string, QueryHelper};
use crate::api::transaction::map_transaction;
use crate::NetworkValidator;
use rosetta::models::*;
use std::cmp::max;

#[derive(Clone)]
pub struct BlockApi {
    network_validator: NetworkValidator,
    query_helper: QueryHelper,
}

impl BlockApi {
    pub fn new(network_validator: NetworkValidator, query_helper: QueryHelper) -> Self {
        Self {
            network_validator,
            query_helper,
        }
    }

    pub async fn block(&self, req: BlockRequest) -> ApiResult<BlockResponse> {
        let block_info = self
            .query_helper
            .query_block_info(Some(req.block_identifier))
            .await?;
        let block_summary = self
            .query_helper
            .client
            .clone()
            .get_block_summary(&block_info.block_hash)
            .await?;
        Ok(BlockResponse {
            block: Some(Box::new(Block {
                block_identifier: Box::new(BlockIdentifier {
                    index: block_info.block_height.height as i64,
                    hash: block_info.block_hash.to_string(),
                }),
                parent_block_identifier: Box::new(BlockIdentifier {
                    index: max(block_info.block_height.height as i64 - 1, 0),
                    hash: block_info.block_parent.to_string(),
                }),
                timestamp: block_info.block_slot_time.timestamp_millis(),
                transactions: block_summary
                    .transaction_summaries
                    .iter()
                    .map(self::map_transaction)
                    .collect(),
                metadata: None, // TODO add minting and rewards
            })),
            other_transactions: None, // currently just expanding all transactions inline
        })
    }

    pub async fn block_transaction(
        &self,
        req: BlockTransactionRequest,
    ) -> ApiResult<BlockTransactionResponse> {
        // TODO Should verify that index is correct?
        let hash = block_hash_from_string(req.block_identifier.hash.as_str())?;
        let block_summary = self
            .query_helper
            .client
            .clone()
            .get_block_summary(&hash) // TODO should probably use the "raw" variant
            .await?;
        match block_summary
            .transaction_summaries
            .iter()
            .find(|t| t.hash.to_string() == req.transaction_identifier.hash)
        {
            None => Err(ApiError::NoTransactionsMatched),
            Some(transaction) => Ok(BlockTransactionResponse {
                transaction: Box::new(map_transaction(transaction)),
            }),
        }
    }
}
