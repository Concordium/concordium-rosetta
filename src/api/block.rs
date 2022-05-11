use std::borrow::BorrowMut;
use crate::{
    api::{
        amount::amount_from_uccd,
        error::{ApiError, ApiResult},
        query::{block_hash_from_string, QueryHelper},
        transaction::*,
    },
    NetworkValidator,
};
use concordium_rust_sdk::types::{AccountStakingInfo, BakerPoolInfo, BlockSummary, DelegationTarget, SpecialTransactionOutcome};
use rosetta::models::*;
use std::cmp::max;
use concordium_rust_sdk::endpoints::Client;
use concordium_rust_sdk::types::hashes::BlockHash;

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
                block_transactions(block_summary, &block_info.block_hash, self.query_helper.client.clone().borrow_mut()),
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

fn block_transactions(block_summary: BlockSummary, block_hash: &BlockHash, client: &mut Client) -> Vec<Transaction> {
    // Synthetic transaction that contains all the minting and rewards operations.
    // Inspired by the "coinbase" transaction in Bitcoin.
    let tokenomics_transaction = Transaction::new(
        TransactionIdentifier::new(TRANSACTION_HASH_TOKENOMICS.to_string()),
        tokenomics_transaction_operations(&block_summary block_hash, client),
    );
    let mut res = vec![tokenomics_transaction];
    res.extend(block_summary.transaction_summaries().iter().map(map_transaction));
    res
}

fn tokenomics_transaction_operations(block_summary: &BlockSummary, block_hash: &BlockHash, client: &mut Client) -> Vec<Operation> {
    let mut index_offset: i64 = 0;
    let next_index = |offset: &mut i64| {
        let res = *offset;
        *offset += 1;
        res
    };
    let mut res = vec![];
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
                        ACCOUNT_BAKING_REWARD.to_string(),
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
                        ACCOUNT_FINALIZATION_REWARD.to_string(),
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
                transaction_fees,
                old_gas_account,
                new_gas_account,
            } => {
                // Could add transaction fees going into GAS account and then extract block
                // rewards, but it seems unnecessary?
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
                remainder,
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
                        ACCOUNT_BAKING_REWARD.to_string(),
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
                        ACCOUNT_FINALIZATION_REWARD.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(
                        -finalization_reward_sum,
                    ))),
                    coin_change:          None,
                    metadata:             None,
                })
            }
            SpecialTransactionOutcome::PaydayFoundationReward {
                foundation_account,
                development_charge,
            } => res.push(Operation {
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
            }),
            SpecialTransactionOutcome::PaydayAccountReward {
                account,
                transaction_fees,
                baker_reward,
                finalization_reward,
            } => {
                if transaction_fees.microccd != 0 {
                    // Query delegation pool of account. Would ideally be included in the outcome but currently isn't.
                    let account_info = client.get_account_info(account, block_hash).await?;
                    let pool_account_address = match account_info.account_stake {
                        None => "".to_string(),
                        Some(staking_info) => ACCOUNT_ACCRUED_POOL_PREFIX + (match staking_info {
                            AccountStakingInfo::Baker { baker_info, .. } => baker_info.baker_id.to_string(),
                            AccountStakingInfo::Delegated { delegation_target, .. } => match delegation_target {
                                DelegationTarget::Passive => POOL_PASSIVE,
                                DelegationTarget::Baker { baker_id } => baker_id.to_string(),
                            },
                        }),
                    };
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations: None,
                        _type: OPERATION_TYPE_PAYDAY_TRANSACTION_FEES_REWARD.to_string(),
                        status: Some(OPERATION_STATUS_OK.to_string()),
                        account: Some(Box::new(AccountIdentifier::new(
                            account.to_string(),
                        ))),
                        amount: Some(Box::new(amount_from_uccd(
                            transaction_fees.microccd as i128,
                        ))),
                        coin_change: None,
                        metadata: None,
                    });
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations: None,
                        _type: OPERATION_TYPE_PAYDAY_TRANSACTION_FEES_REWARD.to_string(),
                        status: Some(OPERATION_STATUS_OK.to_string()),
                        account: Some(Box::new(AccountIdentifier::new(
                            "TODO-correct-accrued-pool-account".to_string(),
                        ))),
                        amount: Some(Box::new(amount_from_uccd(
                            -(transaction_fees.microccd as i128),
                        ))),
                        coin_change: None,
                        metadata: None,
                    });
                }
                if baker_reward.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations: None,
                        _type: OPERATION_TYPE_PAYDAY_BAKER_REWARD.to_string(),
                        status: Some(OPERATION_STATUS_OK.to_string()),
                        account: Some(Box::new(AccountIdentifier::new(
                            account.to_string(),
                        ))),
                        amount: Some(Box::new(amount_from_uccd(
                            baker_reward.microccd as i128,
                        ))),
                        coin_change: None,
                        metadata: None,
                    });
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations: None,
                        _type: OPERATION_TYPE_PAYDAY_BAKER_REWARD.to_string(),
                        status: Some(OPERATION_STATUS_OK.to_string()),
                        account: Some(Box::new(AccountIdentifier::new(
                            ACCOUNT_BAKING_REWARD.to_string(),
                        ))),
                        amount: Some(Box::new(amount_from_uccd(
                            -(baker_reward.microccd as i128),
                        ))),
                        coin_change: None,
                        metadata: None,
                    });
                }
                if finalization_reward != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations: None,
                        _type: OPERATION_TYPE_PAYDAY_FINALIZATION_REWARD.to_string(),
                        status: Some(OPERATION_STATUS_OK.to_string()),
                        account: Some(Box::new(AccountIdentifier::new(
                            account.to_string(),
                        ))),
                        amount: Some(Box::new(amount_from_uccd(
                            finalization_reward.microccd as i128,
                        ))),
                        coin_change: None,
                        metadata: None,
                    });
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations: None,
                        _type: OPERATION_TYPE_PAYDAY_FINALIZATION_REWARD.to_string(),
                        status: Some(OPERATION_STATUS_OK.to_string()),
                        account: Some(Box::new(AccountIdentifier::new(
                            ACCOUNT_FINALIZATION_REWARD.to_string(),
                        ))),
                        amount: Some(Box::new(amount_from_uccd(
                            -(finalization_reward.microccd as i128),
                        ))),
                        coin_change: None,
                        metadata: None,
                    });
                }
            }
            SpecialTransactionOutcome::BlockAccrueReward {
                transaction_fees,
                old_gas_account,
                new_gas_account,
                baker_reward,
                passive_reward,
                foundation_charge,
                baker_id,
            } => {
                if foundation_charge.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations: None,
                        _type: OPERATION_TYPE_PAYDAY_BAKER_REWARD.to_string(),
                        status: Some(OPERATION_STATUS_OK.to_string()),
                        account: Some(Box::new(AccountIdentifier::new(
                            ACCOUNT_ACCRUED_FOUNDATION.to_string(),
                        ))),
                        amount: Some(Box::new(amount_from_uccd(
                            foundation_charge.microccd as i128,
                        ))),
                        coin_change: None,
                        metadata: None,
                    });
                }
                if transaction_fees.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations: None,
                        _type: OPERATION_TYPE_PAYDAY_BAKER_REWARD.to_string(),
                        status: Some(OPERATION_STATUS_OK.to_string()),
                        account: Some(Box::new(AccountIdentifier::new(
                            "TODO-correct-delegation-account".to_string(),
                        ))),
                        amount: Some(Box::new(amount_from_uccd(
                            transaction_fees.microccd as i128,
                        ))),
                        coin_change: None,
                        metadata: None,
                    });
                }
                if baker_reward.microccd != 0 {
                    res.push(Operation {
                        operation_identifier: Box::new(OperationIdentifier::new(next_index(
                            &mut index_offset,
                        ))),
                        related_operations: None,
                        _type: OPERATION_TYPE_PAYDAY_FINALIZATION_REWARD.to_string(),
                        status: Some(OPERATION_STATUS_OK.to_string()),
                        account: Some(Box::new(AccountIdentifier::new(
                            ACCOUNT_FINALIZATION_REWARD.to_string(),
                        ))),
                        amount: Some(Box::new(amount_from_uccd(
                            transaction_fees.microccd as i128,
                        ))),
                        coin_change: None,
                        metadata: None,
                    });
                }
            },
            SpecialTransactionOutcome::PaydayPoolReward {
                pool_owner,
                transaction_fees,
                baker_reward,
                finalization_reward,
            } => (),
        }
    }
    res
}
