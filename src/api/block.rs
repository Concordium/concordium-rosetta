use crate::api::amount::amount_from_uccd;
use crate::api::error::{ApiError, ApiResult};
use crate::api::query::{block_hash_from_string, QueryHelper};
use crate::api::transaction::map_transaction;
use crate::NetworkValidator;
use concordium_rust_sdk::types::{BlockSummary, SpecialTransactionOutcome};
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
                transactions: self::block_transactions(block_summary),
                metadata: None,
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

fn block_transactions(block_summary: BlockSummary) -> Vec<Transaction> {
    // Synthethic transaction that contains all the minting and rewards operations.
    // Inspired by the "coinbase" transaction in Bitcoin.
    let tokenomics_transaction = Transaction {
        transaction_identifier: Box::new(TransactionIdentifier {
            hash: "tokenomics".to_string(),
        }),
        operations: self::tokenomics_transaction_operations(&block_summary),
        related_transactions: None,
        metadata: None,
    };
    let mut res = vec![tokenomics_transaction];
    res.extend(
        block_summary
            .transaction_summaries
            .iter()
            .map(self::map_transaction),
    );
    res
}

fn tokenomics_transaction_operations(block_summary: &BlockSummary) -> Vec<Operation> {
    let mut index_offset: i64 = 0;
    let next_index = |offset: &mut i64| {
        *offset += 1;
        *offset
    };

    let mut res = vec![];
    for e in &block_summary.special_events {
        match e {
            SpecialTransactionOutcome::Mint {
                mint_baking_reward,
                mint_finalization_reward,
                mint_platform_development_charge,
                foundation_account,
            } => res.extend(vec![
                Operation {
                    operation_identifier: Box::new(OperationIdentifier {
                        index: next_index(&mut index_offset),
                        network_index: None,
                    }),
                    related_operations: None,
                    _type: "mint_baking_reward".to_string(),
                    status: None,
                    account: Some(Box::new(AccountIdentifier {
                        address: "baking_reward_account".to_string(),
                        sub_account: None,
                        metadata: None,
                    })),
                    amount: Some(Box::new(amount_from_uccd(
                        mint_baking_reward.microgtu as i64,
                    ))),
                    coin_change: None,
                    metadata: None,
                },
                Operation {
                    operation_identifier: Box::new(OperationIdentifier {
                        index: next_index(&mut index_offset),
                        network_index: None,
                    }),
                    related_operations: None,
                    _type: "mint_finalization_reward".to_string(),
                    status: None,
                    account: Some(Box::new(AccountIdentifier {
                        address: "finalization_reward_account".to_string(),
                        sub_account: None,
                        metadata: None,
                    })),
                    amount: Some(Box::new(amount_from_uccd(
                        mint_finalization_reward.microgtu as i64,
                    ))),
                    coin_change: None,
                    metadata: None,
                },
                Operation {
                    operation_identifier: Box::new(OperationIdentifier {
                        index: next_index(&mut index_offset),
                        network_index: None,
                    }),
                    related_operations: None,
                    _type: "mint_platform_development_charge".to_string(),
                    status: None,
                    account: Some(Box::new(AccountIdentifier {
                        address: foundation_account.to_string(),
                        sub_account: None,
                        metadata: None,
                    })),
                    amount: Some(Box::new(amount_from_uccd(
                        mint_platform_development_charge.microgtu as i64,
                    ))),
                    coin_change: None,
                    metadata: None,
                },
            ]),
            SpecialTransactionOutcome::BlockReward {
                baker_reward,
                baker,
                foundation_charge,
                foundation_account,
                ..
            } => {
                // Could add transaction fees going into GAS account and then extract block rewards, but it seems unnecessary?
                if baker_reward.microgtu != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier {
                            index: next_index(&mut index_offset),
                            network_index: None,
                        }),
                        related_operations: None,
                        _type: "block_reward".to_string(),
                        status: None,
                        account: Some(Box::new(AccountIdentifier {
                            address: baker.to_string(),
                            sub_account: None,
                            metadata: None,
                        })),
                        amount: Some(Box::new(amount_from_uccd(baker_reward.microgtu as i64))),
                        coin_change: None,
                        metadata: None,
                    });
                }
                if foundation_charge.microgtu != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier {
                            index: next_index(&mut index_offset),
                            network_index: None,
                        }),
                        related_operations: None,
                        _type: "block_reward".to_string(),
                        status: None,
                        account: Some(Box::new(AccountIdentifier {
                            address: foundation_account.to_string(),
                            sub_account: None,
                            metadata: None,
                        })),
                        amount: Some(Box::new(amount_from_uccd(
                            foundation_charge.microgtu as i64,
                        ))),
                        coin_change: None,
                        metadata: None,
                    })
                }
            }
            SpecialTransactionOutcome::BakingRewards { baker_rewards, .. } => {
                let mut baking_reward_sum: u64 = 0;
                let mut operation_identifiers = vec![];
                for (baker_account_address, amount) in baker_rewards {
                    baking_reward_sum += amount.microgtu;
                    let id = OperationIdentifier {
                        index: next_index(&mut index_offset),
                        network_index: None,
                    };
                    operation_identifiers.push(id.clone());
                    res.push(Operation {
                        operation_identifier: Box::new(id),
                        related_operations: None,
                        _type: "baking_reward".to_string(),
                        status: None,
                        account: Some(Box::new(AccountIdentifier {
                            address: baker_account_address.to_string(),
                            sub_account: None,
                            metadata: None,
                        })),
                        amount: Some(Box::new(amount_from_uccd(amount.microgtu as i64))),
                        coin_change: None,
                        metadata: None,
                    })
                }
                res.push(Operation {
                    operation_identifier: Box::new(OperationIdentifier {
                        index: next_index(&mut index_offset),
                        network_index: None,
                    }),
                    related_operations: Some(operation_identifiers),
                    _type: "baking_reward".to_string(),
                    status: None,
                    account: Some(Box::new(AccountIdentifier {
                        address: "baking_reward_account".to_string(),
                        sub_account: None,
                        metadata: None,
                    })),
                    amount: Some(Box::new(amount_from_uccd(-(baking_reward_sum as i64)))),
                    coin_change: None,
                    metadata: None,
                })
            }
            SpecialTransactionOutcome::FinalizationRewards {
                finalization_rewards,
                ..
            } => {
                let mut finalization_reward_sum: u64 = 0;
                let mut operation_identifiers = vec![];
                for (baker_account_address, amount) in finalization_rewards {
                    finalization_reward_sum += amount.microgtu;
                    let id = OperationIdentifier {
                        index: next_index(&mut index_offset),
                        network_index: None,
                    };
                    operation_identifiers.push(id.clone());
                    res.push(Operation {
                        operation_identifier: Box::new(id),
                        related_operations: None,
                        _type: "finalization_reward".to_string(),
                        status: None,
                        account: Some(Box::new(AccountIdentifier {
                            address: baker_account_address.to_string(),
                            sub_account: None,
                            metadata: None,
                        })),
                        amount: Some(Box::new(amount_from_uccd(amount.microgtu as i64))),
                        coin_change: None,
                        metadata: None,
                    })
                }
                res.push(Operation {
                    operation_identifier: Box::new(OperationIdentifier {
                        index: next_index(&mut index_offset),
                        network_index: None,
                    }),
                    related_operations: Some(operation_identifiers),
                    _type: "finalization_reward".to_string(),
                    status: None,
                    account: Some(Box::new(AccountIdentifier {
                        address: "finalization_reward_account".to_string(),
                        sub_account: None,
                        metadata: None,
                    })),
                    amount: Some(Box::new(amount_from_uccd(
                        -(finalization_reward_sum as i64),
                    ))),
                    coin_change: None,
                    metadata: None,
                })
            }
        }
    }
    res
}
