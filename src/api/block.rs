use crate::{
    api::{
        amount::amount_from_uccd,
        error::{ApiError, ApiResult},
        query::QueryHelper,
        transaction::*,
    },
    NetworkValidator,
};
use concordium_rust_sdk::{
    common::SerdeSerialize,
    types::{BakerId, SpecialTransactionOutcome, TransactionStatus},
    v2::IntoBlockIdentifier,
};
use futures::{stream::StreamExt, TryStreamExt};
use rosetta::models::*;
use std::cmp::max;

#[derive(Clone)]
pub struct BlockApi {
    network_validator: NetworkValidator,
    query_helper:      QueryHelper,
}

#[derive(SerdeSerialize)]
struct BlockMetadata {
    baker_id: Option<BakerId>,
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
        self.network_validator.validate_network_identifier(*req.network_identifier)?;

        Ok(BlockResponse {
            block:              Some(Box::new(Block {
                block_identifier:        Box::new(BlockIdentifier::new(
                    block_info.block_height.height as i64,
                    block_info.block_hash.to_string(),
                )),
                parent_block_identifier: Box::new(BlockIdentifier::new(
                    max(block_info.block_height.height as i64 - 1, 0),
                    block_info.block_parent.to_string(),
                )),
                timestamp:               block_info.block_slot_time.timestamp_millis(),
                transactions:            self.block_transactions(block_info.block_hash).await?,
                metadata:                Some(
                    serde_json::to_value(&BlockMetadata {
                        baker_id: block_info.block_baker,
                    })
                    .unwrap(),
                ),
            })),
            other_transactions: None, // currently just expanding all transactions inline
        })
    }

    pub async fn block_transaction(
        &self,
        req: BlockTransactionRequest,
    ) -> ApiResult<BlockTransactionResponse> {
        let tx_status =
            self.query_helper.query_transaction_status(req.transaction_identifier.hash).await?;
        match tx_status {
            TransactionStatus::Finalized(finalized_status) => {
                let tx = finalized_status
                    .iter()
                    .next()
                    .ok_or_else(|| {
                        ApiError::InternalServerError(anyhow::anyhow!(
                            "Claimed finalized block is empty."
                        ))
                    })?
                    .1;
                Ok(BlockTransactionResponse::new(map_transaction(tx.to_owned())))
            }
            _ => Err(ApiError::NoTransactionsMatched),
        }
    }

    async fn block_transactions(
        &self,
        block_id: impl IntoBlockIdentifier + Clone,
    ) -> ApiResult<Vec<Transaction>> {
        // Synthetic transaction that contains all the minting and rewards operations.
        // Inspired by the "coinbase" transaction in Bitcoin.
        let tokenomics_transaction = Transaction::new(
            TransactionIdentifier::new(TRANSACTION_HASH_TOKENOMICS.to_string()),
            self.tokenomics_transaction_operations(block_id.clone()).await?,
        );
        let summaries = self.query_helper.query_block_item_summary(block_id).await?;
        let transactions: Vec<Transaction> =
            summaries.map_ok(map_transaction).try_collect().await?;

        let mut res = vec![tokenomics_transaction];
        res.extend(transactions);
        Ok(res)
    }

    async fn tokenomics_transaction_operations(
        &self,
        block_id: impl IntoBlockIdentifier,
    ) -> ApiResult<Vec<Operation>> {
        let mut index_offset: i64 = 0;
        let next_index = |offset: &mut i64| {
            let res = *offset;
            *offset += 1;
            res
        };
        let mut res = vec![];
        let mut current_pool_owner = None;

        let mut special_events = self.query_helper.query_block_special_events(block_id).await?;

        while let Some(e) = special_events.next().await.transpose()? {
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
                            mint_baking_reward.micro_ccd() as i128,
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
                            mint_finalization_reward.micro_ccd() as i128,
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
                            mint_platform_development_charge.micro_ccd() as i128,
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
                    if baker_reward.micro_ccd() != 0 {
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
                                baker_reward.micro_ccd() as i128,
                            ))),
                            coin_change:          None,
                            metadata:             None,
                        });
                    }
                    if foundation_charge.micro_ccd() != 0 {
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
                                foundation_charge.micro_ccd() as i128,
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
                        baking_reward_sum += amount.micro_ccd() as i128;
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
                                amount.micro_ccd() as i128,
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
                        finalization_reward_sum += amount.micro_ccd() as i128;
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
                                amount.micro_ccd() as i128,
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
                    current_pool_owner = pool_owner;
                }
                SpecialTransactionOutcome::PaydayAccountReward {
                    account,
                    transaction_fees,
                    baker_reward,
                    finalization_reward,
                } => {
                    if transaction_fees.micro_ccd() != 0 {
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
                                transaction_fees.micro_ccd() as i128,
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
                                -(transaction_fees.micro_ccd() as i128),
                            ))),
                            coin_change:          None,
                            metadata:             None,
                        });
                    }
                    if baker_reward.micro_ccd() != 0 {
                        res.push(Operation {
                            operation_identifier: Box::new(OperationIdentifier::new(next_index(
                                &mut index_offset,
                            ))),
                            related_operations:   None,
                            _type:                OPERATION_TYPE_PAYDAY_BAKING_REWARD.to_string(),
                            status:               Some(OPERATION_STATUS_OK.to_string()),
                            account:              Some(Box::new(AccountIdentifier::new(
                                account.to_string(),
                            ))),
                            amount:               Some(Box::new(amount_from_uccd(
                                baker_reward.micro_ccd() as i128,
                            ))),
                            coin_change:          None,
                            metadata:             None,
                        });
                        res.push(Operation {
                            operation_identifier: Box::new(OperationIdentifier::new(next_index(
                                &mut index_offset,
                            ))),
                            related_operations:   None,
                            _type:                OPERATION_TYPE_PAYDAY_BAKING_REWARD.to_string(),
                            status:               Some(OPERATION_STATUS_OK.to_string()),
                            account:              Some(Box::new(AccountIdentifier::new(
                                ACCOUNT_REWARD_BAKING.to_string(),
                            ))),
                            amount:               Some(Box::new(amount_from_uccd(
                                -(baker_reward.micro_ccd() as i128),
                            ))),
                            coin_change:          None,
                            metadata:             None,
                        });
                    }
                    if finalization_reward.micro_ccd() != 0 {
                        res.push(Operation {
                            operation_identifier: Box::new(OperationIdentifier::new(next_index(
                                &mut index_offset,
                            ))),
                            related_operations:   None,
                            _type:                OPERATION_TYPE_PAYDAY_FINALIZATION_REWARD
                                .to_string(),
                            status:               Some(OPERATION_STATUS_OK.to_string()),
                            account:              Some(Box::new(AccountIdentifier::new(
                                account.to_string(),
                            ))),
                            amount:               Some(Box::new(amount_from_uccd(
                                finalization_reward.micro_ccd() as i128,
                            ))),
                            coin_change:          None,
                            metadata:             None,
                        });
                        res.push(Operation {
                            operation_identifier: Box::new(OperationIdentifier::new(next_index(
                                &mut index_offset,
                            ))),
                            related_operations:   None,
                            _type:                OPERATION_TYPE_PAYDAY_FINALIZATION_REWARD
                                .to_string(),
                            status:               Some(OPERATION_STATUS_OK.to_string()),
                            account:              Some(Box::new(AccountIdentifier::new(
                                ACCOUNT_REWARD_FINALIZATION.to_string(),
                            ))),
                            amount:               Some(Box::new(amount_from_uccd(
                                -(finalization_reward.micro_ccd() as i128),
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
                            development_charge.micro_ccd() as i128,
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
                            -(development_charge.micro_ccd() as i128),
                        ))),
                        coin_change:          None,
                        metadata:             None,
                    });
                }
                SpecialTransactionOutcome::BlockAccrueReward {
                    baker_reward,
                    passive_reward,
                    foundation_charge,
                    baker_id,
                    ..
                } => {
                    // TODO Add gas account operations.
                    if foundation_charge.micro_ccd() != 0 {
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
                                foundation_charge.micro_ccd() as i128,
                            ))),
                            coin_change:          None,
                            metadata:             None,
                        });
                    }
                    if passive_reward.micro_ccd() != 0 {
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
                                passive_reward.micro_ccd() as i128,
                            ))),
                            coin_change:          None,
                            metadata:             None,
                        });
                    }
                    if baker_reward.micro_ccd() != 0 {
                        res.push(Operation {
                            operation_identifier: Box::new(OperationIdentifier::new(next_index(
                                &mut index_offset,
                            ))),
                            related_operations:   None,
                            _type:                OPERATION_TYPE_BLOCK_ACCRUE_REWARD.to_string(),
                            status:               Some(OPERATION_STATUS_OK.to_string()),
                            account:              Some(Box::new(AccountIdentifier::new(format!(
                                "{}{}",
                                ACCOUNT_ACCRUE_POOL_PREFIX, baker_id
                            )))),
                            amount:               Some(Box::new(amount_from_uccd(
                                baker_reward.micro_ccd() as i128,
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
}
