use crate::{
    api::{
        amount::amount_from_uccd,
        error::{ApiError, ApiResult},
        query::{block_hash_from_string, QueryHelper},
        transaction::*,
    },
    NetworkValidator,
};
use concordium_rust_sdk::types::{BlockSummary, SpecialTransactionOutcome};
use rosetta::models::*;
use std::cmp::max;

#[derive(Clone)]
pub struct BlockApi {
    network_validator: NetworkValidator,
    query_helper:      QueryHelper,
}

impl BlockApi {
    pub fn new(network_validator: NetworkValidator, query_helper: QueryHelper) -> Self {
        Self {
            network_validator,
            query_helper,
        }
    }

    pub async fn block(&self, req: BlockRequest) -> ApiResult<BlockResponse> {
        let block_info = self.query_helper.query_block_info(Some(req.block_identifier)).await?;
        let block_summary =
            self.query_helper.client.clone().get_block_summary(&block_info.block_hash).await?;
        self.network_validator.validate_network_identifier(*req.network_identifier)?;
        Ok(BlockResponse {
            block:              Some(Box::new(Block::new(
                BlockIdentifier::new(
                    block_info.block_height.height as i64,
                    block_info.block_hash.to_string(),
                ),
                BlockIdentifier::new(
                    max(block_info.block_height.height as i64 - 1, 0),
                    block_info.block_parent.to_string(),
                ),
                block_info.block_slot_time.timestamp_millis(),
                block_transactions(block_summary).await?,
            ))),
            other_transactions: None, // currently just expanding all transactions inline
        })
    }

    pub async fn block_transaction(
        &self,
        req: BlockTransactionRequest,
    ) -> ApiResult<BlockTransactionResponse> {
        let hash = block_hash_from_string(req.block_identifier.hash.as_str())?;
        let block_summary = self
            .query_helper
            .client
            .clone()
            .get_block_summary(&hash) // TODO should probably use the "raw" variant
            .await?;
        match block_summary
            .transaction_summaries()
            .iter()
            .find(|t| t.hash.to_string() == req.transaction_identifier.hash)
        {
            None => Err(ApiError::NoTransactionsMatched),
            Some(transaction) => Ok(BlockTransactionResponse::new(map_transaction(transaction))),
        }
    }
}

async fn block_transactions(block_summary: BlockSummary) -> ApiResult<Vec<Transaction>> {
    // Synthetic transaction that contains all the minting and rewards operations.
    // Inspired by the "coinbase" transaction in Bitcoin.
    let tokenomics_transaction = Transaction::new(
        TransactionIdentifier::new(TRANSACTION_HASH_TOKENOMICS.to_string()),
        tokenomics_transaction_operations(&block_summary).await?,
    );
    let mut res = vec![tokenomics_transaction];
    res.extend(block_summary.transaction_summaries().iter().map(map_transaction));
    Ok(res)
}

async fn tokenomics_transaction_operations(
    block_summary: &BlockSummary,
) -> ApiResult<Vec<Operation>> {
    let mut index_offset: i64 = 0;
    let next_index = |offset: &mut i64| {
        let res = *offset;
        *offset += 1;
        res
    };
    let mut res = vec![];
    let mut current_pool_owner = None;
    for e in block_summary.special_events() {
        match e {
            SpecialTransactionOutcome::Mint {
                mint_baking_reward,
                mint_finalization_reward,
                mint_platform_development_charge,
                foundation_account,
            } => res.extend(vec![
                Operation {
                    operation_identifier: Box::new(OperationIdentifier::new(next_index(
                        &mut index_offset,
                    ))),
                    related_operations:   None,
                    _type:                OPERATION_TYPE_MINT_BAKING_REWARD.to_string(),
                    status:               Some(OPERATION_STATUS_OK.to_string()),
                    account:              Some(Box::new(AccountIdentifier::new(
                        ACCOUNT_REWARD_BAKING.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(
                        mint_baking_reward.microccd as i128,
                    ))),
                    coin_change:          None,
                    metadata:             None,
                },
                Operation {
                    operation_identifier: Box::new(OperationIdentifier::new(next_index(
                        &mut index_offset,
                    ))),
                    related_operations:   None,
                    _type:                OPERATION_TYPE_MINT_FINALIZATION_REWARD.to_string(),
                    status:               Some(OPERATION_STATUS_OK.to_string()),
                    account:              Some(Box::new(AccountIdentifier::new(
                        ACCOUNT_REWARD_FINALIZATION.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(
                        mint_finalization_reward.microccd as i128,
                    ))),
                    coin_change:          None,
                    metadata:             None,
                },
                Operation {
                    operation_identifier: Box::new(OperationIdentifier::new(next_index(
                        &mut index_offset,
                    ))),
                    related_operations:   None,
                    _type:                OPERATION_TYPE_MINT_PLATFORM_DEVELOPMENT_CHARGE
                        .to_string(),
                    status:               Some(OPERATION_STATUS_OK.to_string()),
                    account:              Some(Box::new(AccountIdentifier::new(
                        foundation_account.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(
                        mint_platform_development_charge.microccd as i128,
                    ))),
                    coin_change:          None,
                    metadata:             None,
                },
            ]),
            SpecialTransactionOutcome::BlockReward {
                baker_reward,
                baker,
                foundation_charge,
                foundation_account,
                ..
            } => {
                // TODO Add gas account operations.
                if baker_reward.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_BLOCK_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            baker.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            baker_reward.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                }
                if foundation_charge.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_BLOCK_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            foundation_account.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            foundation_charge.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    })
                }
            }
            SpecialTransactionOutcome::BakingRewards {
                baker_rewards,
                ..
            } => {
                let mut baking_reward_sum: i128 = 0;
                let mut operation_identifiers = vec![];
                for (baker_account_address, amount) in baker_rewards {
                    baking_reward_sum += amount.microccd as i128;
                    let id = OperationIdentifier::new(next_index(&mut index_offset));
                    operation_identifiers.push(id.clone());
                    res.push(Operation {
                        operation_identifier: Box::new(id),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_BAKING_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            baker_account_address.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            amount.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    })
                }
                res.push(Operation {
                    operation_identifier: Box::new(OperationIdentifier::new(next_index(
                        &mut index_offset,
                    ))),
                    related_operations:   Some(operation_identifiers),
                    _type:                OPERATION_TYPE_BAKING_REWARD.to_string(),
                    status:               Some(OPERATION_STATUS_OK.to_string()),
                    account:              Some(Box::new(AccountIdentifier::new(
                        ACCOUNT_REWARD_BAKING.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(-baking_reward_sum))),
                    coin_change:          None,
                    metadata:             None,
                })
            }
            SpecialTransactionOutcome::FinalizationRewards {
                finalization_rewards,
                ..
            } => {
                let mut finalization_reward_sum: i128 = 0;
                let mut operation_identifiers = vec![];
                for (baker_account_address, amount) in finalization_rewards {
                    finalization_reward_sum += amount.microccd as i128;
                    let id = OperationIdentifier {
                        index:         next_index(&mut index_offset),
                        network_index: None,
                    };
                    operation_identifiers.push(id.clone());
                    res.push(Operation {
                        operation_identifier: Box::new(id),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_FINALIZATION_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            baker_account_address.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            amount.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    })
                }
                res.push(Operation {
                    operation_identifier: Box::new(OperationIdentifier::new(next_index(
                        &mut index_offset,
                    ))),
                    related_operations:   Some(operation_identifiers),
                    _type:                OPERATION_TYPE_FINALIZATION_REWARD.to_string(),
                    status:               Some(OPERATION_STATUS_OK.to_string()),
                    account:              Some(Box::new(AccountIdentifier::new(
                        ACCOUNT_REWARD_FINALIZATION.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(
                        -finalization_reward_sum,
                    ))),
                    coin_change:          None,
                    metadata:             None,
                })
            }
            SpecialTransactionOutcome::PaydayPoolReward {
                pool_owner,
                ..
            } => {
                // The events are ordered such that PaydayPoolReward events are followed
                // by PaydayAccountReward events for the accounts in the given pool.
                current_pool_owner = *pool_owner;
            }
            SpecialTransactionOutcome::PaydayAccountReward {
                account,
                transaction_fees,
                baker_reward,
                finalization_reward,
            } => {
                if transaction_fees.microccd != 0 {
                    let pool_account_address =
                        format!("{}{}", ACCOUNT_ACCRUE_POOL_PREFIX, match current_pool_owner {
                            None => POOL_PASSIVE.to_string(),
                            Some(id) => id.to_string(),
                        });
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_PAYDAY_TRANSACTION_FEES_REWARD
                            .to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            account.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            transaction_fees.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_PAYDAY_TRANSACTION_FEES_REWARD
                            .to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            pool_account_address,
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            -(transaction_fees.microccd as i128),
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                }
                if baker_reward.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_PAYDAY_BAKER_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            account.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            baker_reward.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_PAYDAY_BAKER_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            ACCOUNT_REWARD_BAKING.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            -(baker_reward.microccd as i128),
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                }
                if finalization_reward.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_PAYDAY_FINALIZATION_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            account.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            finalization_reward.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_PAYDAY_FINALIZATION_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            ACCOUNT_REWARD_FINALIZATION.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            -(finalization_reward.microccd as i128),
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                }
            }
            SpecialTransactionOutcome::PaydayFoundationReward {
                foundation_account,
                development_charge,
            } => {
                res.push(Operation {
                    operation_identifier: Box::new(OperationIdentifier::new(next_index(
                        &mut index_offset,
                    ))),
                    related_operations:   None,
                    _type:                OPERATION_TYPE_PAYDAY_FOUNDATION_REWARD.to_string(),
                    status:               Some(OPERATION_STATUS_OK.to_string()),
                    account:              Some(Box::new(AccountIdentifier::new(
                        foundation_account.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(
                        development_charge.microccd as i128,
                    ))),
                    coin_change:          None,
                    metadata:             None,
                });
                res.push(Operation {
                    operation_identifier: Box::new(OperationIdentifier::new(next_index(
                        &mut index_offset,
                    ))),
                    related_operations:   None,
                    _type:                OPERATION_TYPE_PAYDAY_FOUNDATION_REWARD.to_string(),
                    status:               Some(OPERATION_STATUS_OK.to_string()),
                    account:              Some(Box::new(AccountIdentifier::new(
                        ACCOUNT_ACCRUE_FOUNDATION.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(
                        -(development_charge.microccd as i128),
                    ))),
                    coin_change:          None,
                    metadata:             None,
                });
            }
            SpecialTransactionOutcome::BlockAccrueReward {
                transaction_fees,
                baker_reward,
                passive_reward,
                foundation_charge,
                baker_id,
                ..
            } => {
                // TODO Add gas account operations.
                if foundation_charge.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_BLOCK_ACCRUE_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            ACCOUNT_ACCRUE_FOUNDATION.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            foundation_charge.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                }
                if transaction_fees.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_BLOCK_ACCRUE_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(format!(
                            "{}{}",
                            ACCOUNT_ACCRUE_POOL_PREFIX,
                            baker_id.to_string()
                        )))),
                        amount:               Some(Box::new(amount_from_uccd(
                            transaction_fees.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                }
                if passive_reward.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_BLOCK_ACCRUE_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(format!(
                            "{}{}",
                            ACCOUNT_ACCRUE_POOL_PREFIX, POOL_PASSIVE
                        )))),
                        amount:               Some(Box::new(amount_from_uccd(
                            passive_reward.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                }
                if baker_reward.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations:   None,
                        _type:                OPERATION_TYPE_BLOCK_ACCRUE_REWARD.to_string(),
                        status:               Some(OPERATION_STATUS_OK.to_string()),
                        account:              Some(Box::new(AccountIdentifier::new(
                            ACCOUNT_REWARD_BAKING.to_string(),
                        ))),
                        amount:               Some(Box::new(amount_from_uccd(
                            baker_reward.microccd as i128,
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                }
            }
        }
    }
    Ok(res)
}
