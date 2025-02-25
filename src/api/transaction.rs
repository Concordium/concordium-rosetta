use crate::api::{
    amount::amount_from_uccd,
    error::{ApiError, ApiResult},
};
use concordium_rust_sdk::{
    common::{
        types::{Amount, Timestamp, TransactionTime},
        SerdeSerialize,
    },
    constants::EncryptedAmountsCurve,
    encrypted_transfers::types::*,
    id::types::AccountAddress,
    types::*,
};
use rosetta::models::{
    AccountIdentifier, Operation, OperationIdentifier, Transaction, TransactionIdentifier,
};
use serde_json::{Error, Value};
use std::ops::Deref;

#[derive(SerdeSerialize)]
struct TransactionRejectedMetadata {
    reject_reason: RejectReason, // TODO replace with explicit structure
}

#[derive(SerdeSerialize)]
struct ModuleDeployedMetadata {
    module_ref: smart_contracts::ModuleReference,
}

#[derive(SerdeSerialize)]
struct ContractInitializedMetadata {
    module_ref: smart_contracts::ModuleReference,
    address:    ContractAddress,
    init_name:  smart_contracts::OwnedContractName,
    events:     Vec<smart_contracts::ContractEvent>,
}

#[derive(SerdeSerialize)]
// TODO Name "transferred" for consistency?
pub struct MemoMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<Memo>,
}

#[derive(SerdeSerialize)]
struct BakerAddedMetadata {
    baker_id:         BakerId,
    account:          AccountAddress,
    sign_key:         BakerSignatureVerifyKey,
    election_key:     BakerElectionVerifyKey,
    aggregation_key:  BakerAggregationVerifyKey,
    stake_uccd:       concordium_rust_sdk::common::types::Amount,
    restake_earnings: bool,
}

#[derive(SerdeSerialize)]
struct BakerRemovedMetadata {
    baker_id: BakerId,
}

#[derive(SerdeSerialize)]
struct BakerStakeUpdatedMetadata {
    baker_id:       BakerId,
    new_stake_uccd: Amount,
    increased:      bool,
}

#[derive(SerdeSerialize)]
struct BakerRestakeEarningsUpdatedMetadata {
    baker_id:         BakerId,
    restake_earnings: bool,
}

#[derive(SerdeSerialize)]
struct BakerKeysUpdatedMetadata {
    baker_id:        BakerId,
    account:         AccountAddress,
    sign_key:        BakerSignatureVerifyKey,
    election_key:    BakerElectionVerifyKey,
    aggregation_key: BakerAggregationVerifyKey,
}

#[derive(SerdeSerialize)]
struct BakerSetOpenStatusMetadata {
    baker_id:    BakerId,
    open_status: String,
}

#[derive(SerdeSerialize)]
struct BakerSetMetadataUrlMetadata {
    baker_id:     BakerId,
    metadata_url: String,
}

#[derive(SerdeSerialize)]
struct BakerSetTransactionFeeCommissionMetadata {
    baker_id:                   BakerId,
    transaction_fee_commission: String,
}

#[derive(SerdeSerialize)]
struct BakerSetBakingRewardCommissionMetadata {
    baker_id:                 BakerId,
    baking_reward_commission: String,
}

#[derive(SerdeSerialize)]
struct BakerSetFinalizationRewardCommissionMetadata {
    baker_id: BakerId,
    finalization_reward_commission: String,
}

#[derive(SerdeSerialize)]
struct BakerSuspensionMetadata {
    baker_id:  BakerId,
    suspended: bool,
}

#[derive(SerdeSerialize)]
struct DelegationStakeUpdatedMetadata {
    delegator_id:   DelegatorId,
    new_stake_uccd: Amount,
    increased:      bool,
}

#[derive(SerdeSerialize)]
struct DelegationAddedMetadata {
    delegator_id: DelegatorId,
}

#[derive(SerdeSerialize)]
struct DelegationRemovedMetadata {
    delegator_id: DelegatorId,
}

#[derive(SerdeSerialize)]
struct DelegationSetRestakeEarningsMetadata {
    delegator_id:     DelegatorId,
    restake_earnings: bool,
}

#[derive(SerdeSerialize)]
struct DelegationSetDelegationTargetMetadata {
    delegator_id:      DelegatorId,
    delegation_target: String,
}

#[derive(SerdeSerialize)]
struct EncryptedAmountTransferredSenderMetadata {
    new_encrypted_balance: EncryptedAmount<EncryptedAmountsCurve>,
    encrypted_amount:      EncryptedAmount<EncryptedAmountsCurve>,
    up_to_index:           EncryptedAmountAggIndex,
}
#[derive(SerdeSerialize)]
struct EncryptedAmountTransferredReceiverMetadata {
    new_index:        EncryptedAmountIndex,
    encrypted_amount: EncryptedAmount<EncryptedAmountsCurve>,
}

#[derive(SerdeSerialize)]
struct TransferredToEncryptedMetadata {
    new_encrypted_amount: EncryptedAmount<EncryptedAmountsCurve>,
}

#[derive(SerdeSerialize)]
struct TransferredToPublicMetadata {
    address:              AccountAddress,
    new_encrypted_amount: EncryptedAmount<EncryptedAmountsCurve>,
    encrypted_amount:     EncryptedAmount<EncryptedAmountsCurve>,
    up_to_index:          EncryptedAmountAggIndex,
}

#[derive(SerdeSerialize)]
struct TransferredWithScheduleMetadata {
    amounts: Vec<TimestampedAmount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memo:    Option<Memo>,
}

#[derive(SerdeSerialize)]
struct TimestampedAmount {
    timestamp:   Timestamp,
    amount_uccd: Amount,
}

#[derive(SerdeSerialize)]
struct CredentialKeysUpdatedMetadata {
    credential_id: CredentialRegistrationID,
}

#[derive(SerdeSerialize)]
struct CredentialsUpdatedMetadata {
    removed_credential_ids: Vec<CredentialRegistrationID>,
    added_credential_ids:   Vec<CredentialRegistrationID>,
    new_threshold:          AccountThreshold,
}

#[derive(SerdeSerialize)]
struct DataRegisteredMetadata {
    data: RegisteredData,
}

#[derive(SerdeSerialize)]
struct AccountCreatedMetadata {
    credential_type: String,
    address:         AccountAddress,
    registration_id: CredentialRegistrationID,
}

#[derive(SerdeSerialize)]
struct ChainUpdateMetadata {
    effective_time: TransactionTime,
    payload:        UpdatePayload,
}

pub const ACCOUNT_REWARD_BAKING: &str = "baking_reward_account";
pub const ACCOUNT_REWARD_FINALIZATION: &str = "finalization_reward_account";
pub const ACCOUNT_ACCRUE_FOUNDATION: &str = "foundation_accrue_account";
pub const ACCOUNT_ACCRUE_POOL_PREFIX: &str = "pool_accrue_account:";
pub const ACCOUNT_CONTRACT_PREFIX: &str = "contract:";
pub const POOL_PASSIVE: &str = "passive";

pub const OPERATION_STATUS_OK: &str = "ok";
pub const OPERATION_STATUS_FAIL: &str = "fail";

pub const OPERATION_TYPE_UNKNOWN: &str = "unknown";
pub const OPERATION_TYPE_FEE: &str = "fee";

pub const OPERATION_TYPE_ACCOUNT_CREATION: &str = "account_creation";
pub const OPERATION_TYPE_ADD_BAKER: &str = "add_baker";
pub const OPERATION_TYPE_BAKING_REWARD: &str = "baking_reward";
pub const OPERATION_TYPE_BLOCK_REWARD: &str = "block_reward";
pub const OPERATION_TYPE_CHAIN_UPDATE: &str = "chain_update";
pub const OPERATION_TYPE_DEPLOY_MODULE: &str = "deploy_module";
pub const OPERATION_TYPE_ENCRYPTED_AMOUNT_TRANSFER: &str = "encrypted_amount_transfer";
pub const OPERATION_TYPE_FINALIZATION_REWARD: &str = "finalization_reward";
pub const OPERATION_TYPE_INIT_CONTRACT: &str = "init_contract";
pub const OPERATION_TYPE_MINT_BAKING_REWARD: &str = "mint_baking_reward";
pub const OPERATION_TYPE_MINT_FINALIZATION_REWARD: &str = "mint_finalization_reward";
pub const OPERATION_TYPE_MINT_PLATFORM_DEVELOPMENT_CHARGE: &str =
    "mint_platform_development_charge";
pub const OPERATION_TYPE_REGISTER_DATA: &str = "register_data";
pub const OPERATION_TYPE_REMOVE_BAKER: &str = "remove_baker";
pub const OPERATION_TYPE_TRANSFER: &str = "transfer";
pub const OPERATION_TYPE_TRANSFER_TO_ENCRYPTED: &str = "transfer_to_encrypted";
pub const OPERATION_TYPE_TRANSFER_TO_PUBLIC: &str = "transfer_to_public";
pub const OPERATION_TYPE_TRANSFER_WITH_SCHEDULE: &str = "transfer_with_schedule"; // TODO is just a transfer with schedule metadata?
pub const OPERATION_TYPE_UPDATE_BAKER_KEYS: &str = "update_baker_keys";
pub const OPERATION_TYPE_UPDATE_BAKER_RESTAKE_EARNINGS: &str = "update_baker_restake_earnings";
pub const OPERATION_TYPE_UPDATE_BAKER_STAKE: &str = "update_baker_stake";
pub const OPERATION_TYPE_UPDATE_CONTRACT: &str = "update_contract";
pub const OPERATION_TYPE_UPDATE_CREDENTIAL_KEYS: &str = "update_credential_keys";
pub const OPERATION_TYPE_UPDATE_CREDENTIALS: &str = "update_credentials";
pub const OPERATION_TYPE_PAYDAY_FOUNDATION_REWARD: &str = "payday_foundation_reward";
pub const OPERATION_TYPE_PAYDAY_TRANSACTION_FEES_REWARD: &str = "payday_transaction_fees_reward";
pub const OPERATION_TYPE_PAYDAY_BAKING_REWARD: &str = "payday_baking_reward";
pub const OPERATION_TYPE_PAYDAY_FINALIZATION_REWARD: &str = "payday_finalization_reward";
pub const OPERATION_TYPE_BLOCK_ACCRUE_REWARD: &str = "block_accrue_reward";
pub const OPERATION_TYPE_CONFIGURE_BAKER: &str = "configure_baker";
pub const OPERATION_TYPE_CONFIGURE_DELEGATION: &str = "configure_delegation";
pub const OPERATION_TYPE_VALIDATOR_PRIMED_FOR_SUSPENSION: &str = "validator_primed_for_suspension";
pub const OPERATION_TYPE_VALIDATOR_SUSPENDED: &str = "validator_suspended";

pub const TRANSACTION_HASH_TOKENOMICS: &str = "tokenomics";

pub fn contract_address_string(contract_addr: &ContractAddress) -> String {
    format!("{}{}_{}", ACCOUNT_CONTRACT_PREFIX, contract_addr.index, contract_addr.subindex)
}

pub fn map_transaction(info: BlockItemSummary) -> Transaction {
    let (operations, extra_metadata) = match &info.details {
        BlockItemSummaryDetails::AccountTransaction(details) => {
            let (ops, metadata) = operations_and_metadata_from_account_transaction_details(details);
            let mut ops_with_fee = ops.clone();
            if details.cost.micro_ccd() != 0 {
                ops_with_fee.push(Operation {
                    operation_identifier: Box::new(OperationIdentifier::new(ops.len() as i64)),
                    related_operations:   None,
                    _type:                OPERATION_TYPE_FEE.to_string(),
                    status:               Some(OPERATION_STATUS_OK.to_string()),
                    account:              Some(Box::new(AccountIdentifier::new(
                        details.sender.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(
                        -(details.cost.micro_ccd() as i128),
                    ))),
                    coin_change:          None,
                    metadata:             None,
                });
            }
            (ops_with_fee, metadata)
        }
        BlockItemSummaryDetails::AccountCreation(details) => {
            (operations_and_metadata_from_account_creation_details(details), None)
        }
        BlockItemSummaryDetails::Update(details) => {
            (operations_and_metadata_from_chain_update_details(details), None)
        }
    };
    Transaction {
        transaction_identifier: Box::new(TransactionIdentifier {
            hash: info.hash.to_string(),
        }),
        operations,
        related_transactions: None,
        metadata: extra_metadata.map(Result::unwrap),
    }
}

fn operations_and_metadata_from_account_transaction_details(
    details: &AccountTransactionDetails,
) -> (Vec<Operation>, Option<Result<Value, Error>>) {
    match &details.effects {
        AccountTransactionEffects::None {
            transaction_type,
            reject_reason,
        } => (
            vec![Operation {
                operation_identifier: Box::new(OperationIdentifier::new(0)),
                related_operations:   None,
                _type:                transaction_type_to_operation_type(*transaction_type),
                status:               Some(OPERATION_STATUS_FAIL.to_string()),
                account:              Some(Box::new(AccountIdentifier::new(
                    details.sender.to_string(),
                ))),
                amount:               None,
                coin_change:          None,
                metadata:             Some(
                    serde_json::to_value(&TransactionRejectedMetadata {
                        reject_reason: reject_reason.clone(),
                    })
                    .unwrap(),
                ),
            }],
            None,
        ),
        AccountTransactionEffects::ModuleDeployed {
            module_ref,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                None,
                Some(&ModuleDeployedMetadata {
                    module_ref: *module_ref,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::ContractInitialized {
            data,
        } => {
            // TODO Adapt and use 'simple_transfer_operations'?
            let mut ops = vec![normal_account_transaction_operation(
                0,
                details,
                Some(amount_from_uccd(-(data.amount.micro_ccd() as i128))),
                Some(&ContractInitializedMetadata {
                    module_ref: data.origin_ref,
                    address:    data.address,
                    init_name:  data.init_name.clone(),
                    events:     data.events.clone(),
                }),
            )];
            if data.amount.micro_ccd() != 0 {
                ops.push(account_transaction_operation::<Value>(
                    1,
                    details,
                    contract_address_string(&data.address),
                    Some(amount_from_uccd(data.amount.micro_ccd() as i128)),
                    None,
                ));
            }
            (ops, None)
        }
        AccountTransactionEffects::ContractUpdateIssued {
            effects,
        } => (contract_update_operations(details, effects), None),
        AccountTransactionEffects::AccountTransfer {
            amount,
            to,
        } => (simple_transfer_operations(details, amount, to), None),
        AccountTransactionEffects::AccountTransferWithMemo {
            amount,
            to,
            memo,
        } => (
            simple_transfer_operations(details, amount, to),
            Some(serde_json::to_value(MemoMetadata {
                memo: Some(memo.clone()),
            })),
        ),
        AccountTransactionEffects::BakerAdded {
            data,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                None,
                Some(&BakerAddedMetadata {
                    baker_id:         data.keys_event.baker_id,
                    account:          data.keys_event.account,
                    sign_key:         data.keys_event.sign_key.clone(),
                    election_key:     data.keys_event.election_key.clone(),
                    aggregation_key:  data.keys_event.aggregation_key.clone(),
                    stake_uccd:       data.stake,
                    restake_earnings: data.restake_earnings,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::BakerRemoved {
            baker_id,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                None,
                Some(&BakerRemovedMetadata {
                    baker_id: *baker_id,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::BakerStakeUpdated {
            data,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                None,
                data.map(|d| BakerStakeUpdatedMetadata {
                    baker_id:       d.baker_id,
                    new_stake_uccd: d.new_stake,
                    increased:      d.increased,
                })
                .as_ref(),
            )],
            None,
        ),
        AccountTransactionEffects::BakerRestakeEarningsUpdated {
            baker_id,
            restake_earnings,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                None,
                Some(&BakerRestakeEarningsUpdatedMetadata {
                    baker_id:         *baker_id,
                    restake_earnings: *restake_earnings,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::BakerKeysUpdated {
            data,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                None,
                Some(&BakerKeysUpdatedMetadata {
                    baker_id:        data.baker_id,
                    account:         data.account,
                    sign_key:        data.sign_key.clone(),
                    election_key:    data.election_key.clone(),
                    aggregation_key: data.aggregation_key.clone(),
                }),
            )],
            None,
        ),
        AccountTransactionEffects::EncryptedAmountTransferred {
            removed,
            added,
        } => (encrypted_transfer_operations(details, removed, added), None),
        AccountTransactionEffects::EncryptedAmountTransferredWithMemo {
            removed,
            added,
            memo,
        } => (
            encrypted_transfer_operations(details, removed, added),
            Some(serde_json::to_value(MemoMetadata {
                memo: Some(memo.clone()),
            })),
        ),
        AccountTransactionEffects::TransferredToEncrypted {
            data,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(amount_from_uccd(-(data.amount.micro_ccd() as i128))),
                Some(&TransferredToEncryptedMetadata {
                    new_encrypted_amount: data.new_amount.clone(),
                }),
            )],
            None,
        ),
        AccountTransactionEffects::TransferredToPublic {
            removed,
            amount,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(amount_from_uccd(amount.micro_ccd() as i128)),
                Some(&TransferredToPublicMetadata {
                    address:              removed.account,
                    new_encrypted_amount: removed.new_amount.clone(),
                    encrypted_amount:     removed.input_amount.clone(),
                    up_to_index:          removed.up_to_index,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::TransferredWithSchedule {
            to,
            amount,
        } => (
            simple_transfer_operations(
                details,
                &Amount::from_micro_ccd(amount.iter().map(|(_, a)| a.micro_ccd()).sum()),
                to,
            ),
            Some(serde_json::to_value(TransferredWithScheduleMetadata {
                amounts: amount
                    .iter()
                    .map(|(t, a)| TimestampedAmount {
                        timestamp:   *t,
                        amount_uccd: *a,
                    })
                    .collect(),
                memo:    None,
            })),
        ),
        AccountTransactionEffects::TransferredWithScheduleAndMemo {
            to,
            amount,
            memo,
        } => (
            simple_transfer_operations(
                details,
                &Amount::from_micro_ccd(amount.iter().map(|(_, a)| a.micro_ccd()).sum()),
                to,
            ),
            Some(serde_json::to_value(TransferredWithScheduleMetadata {
                amounts: amount
                    .iter()
                    .map(|(t, a)| TimestampedAmount {
                        timestamp:   *t,
                        amount_uccd: *a,
                    })
                    .collect(),
                memo:    Some(memo.clone()),
            })),
        ),
        AccountTransactionEffects::CredentialKeysUpdated {
            cred_id,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                None,
                Some(&CredentialKeysUpdatedMetadata {
                    credential_id: *cred_id,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::CredentialsUpdated {
            new_cred_ids,
            removed_cred_ids,
            new_threshold,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                None,
                Some(&CredentialsUpdatedMetadata {
                    removed_credential_ids: removed_cred_ids.clone(),
                    added_credential_ids:   new_cred_ids.clone(),
                    new_threshold:          *new_threshold,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::DataRegistered {
            data,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                None,
                Some(&DataRegisteredMetadata {
                    data: data.clone(),
                }),
            )],
            None,
        ),
        AccountTransactionEffects::BakerConfigured {
            data: events,
        } => (
            events
                .iter()
                .enumerate()
                .map(|(i, event)| match event {
                    BakerEvent::DelegationRemoved {
                        delegator_id,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&DelegationRemovedMetadata {
                            delegator_id: *delegator_id,
                        }),
                    ),
                    BakerEvent::BakerAdded {
                        data,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerAddedMetadata {
                            baker_id:         data.keys_event.baker_id,
                            account:          data.keys_event.account,
                            sign_key:         data.keys_event.sign_key.clone(),
                            election_key:     data.keys_event.election_key.clone(),
                            aggregation_key:  data.keys_event.aggregation_key.clone(),
                            stake_uccd:       data.stake,
                            restake_earnings: data.restake_earnings,
                        }),
                    ),
                    BakerEvent::BakerRemoved {
                        baker_id,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerRemovedMetadata {
                            baker_id: *baker_id,
                        }),
                    ),
                    BakerEvent::BakerStakeIncreased {
                        baker_id,
                        new_stake,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerStakeUpdatedMetadata {
                            baker_id:       *baker_id,
                            new_stake_uccd: *new_stake,
                            increased:      true,
                        }),
                    ),
                    BakerEvent::BakerStakeDecreased {
                        baker_id,
                        new_stake,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerStakeUpdatedMetadata {
                            baker_id:       *baker_id,
                            new_stake_uccd: *new_stake,
                            increased:      false,
                        }),
                    ),
                    BakerEvent::BakerRestakeEarningsUpdated {
                        baker_id,
                        restake_earnings,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerRestakeEarningsUpdatedMetadata {
                            baker_id:         *baker_id,
                            restake_earnings: *restake_earnings,
                        }),
                    ),
                    BakerEvent::BakerKeysUpdated {
                        data,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerKeysUpdatedMetadata {
                            baker_id:        data.baker_id,
                            account:         data.account,
                            sign_key:        data.sign_key.clone(),
                            election_key:    data.election_key.clone(),
                            aggregation_key: data.aggregation_key.clone(),
                        }),
                    ),
                    BakerEvent::BakerSetOpenStatus {
                        baker_id,
                        open_status,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerSetOpenStatusMetadata {
                            baker_id:    *baker_id,
                            open_status: match open_status {
                                OpenStatus::OpenForAll => "open_for_all".to_string(),
                                OpenStatus::ClosedForNew => "closed_for_new".to_string(),
                                OpenStatus::ClosedForAll => "closed_for_all".to_string(),
                            },
                        }),
                    ),
                    BakerEvent::BakerSetMetadataURL {
                        baker_id,
                        metadata_url,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerSetMetadataUrlMetadata {
                            baker_id:     *baker_id,
                            metadata_url: metadata_url.to_string(),
                        }),
                    ),
                    BakerEvent::BakerSetTransactionFeeCommission {
                        baker_id,
                        transaction_fee_commission,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerSetTransactionFeeCommissionMetadata {
                            baker_id:                   *baker_id,
                            transaction_fee_commission: transaction_fee_commission.to_string(),
                        }),
                    ),
                    BakerEvent::BakerSetBakingRewardCommission {
                        baker_id,
                        baking_reward_commission,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerSetBakingRewardCommissionMetadata {
                            baker_id:                 *baker_id,
                            baking_reward_commission: baking_reward_commission.to_string(),
                        }),
                    ),
                    BakerEvent::BakerSetFinalizationRewardCommission {
                        baker_id,
                        finalization_reward_commission,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerSetFinalizationRewardCommissionMetadata {
                            baker_id: *baker_id,
                            finalization_reward_commission: finalization_reward_commission
                                .to_string(),
                        }),
                    ),
                    BakerEvent::BakerSuspended {
                        baker_id,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerSuspensionMetadata {
                            baker_id:  *baker_id,
                            suspended: true,
                        }),
                    ),
                    BakerEvent::BakerResumed {
                        baker_id,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerSuspensionMetadata {
                            baker_id:  *baker_id,
                            suspended: false,
                        }),
                    ),
                })
                .collect(),
            None,
        ),
        AccountTransactionEffects::DelegationConfigured {
            data: events,
        } => (
            events
                .iter()
                .enumerate()
                .map(|(i, event)| match event {
                    DelegationEvent::BakerRemoved {
                        baker_id,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&BakerRemovedMetadata {
                            baker_id: *baker_id,
                        }),
                    ),
                    DelegationEvent::DelegationAdded {
                        delegator_id,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&DelegationAddedMetadata {
                            delegator_id: *delegator_id,
                        }),
                    ),
                    DelegationEvent::DelegationRemoved {
                        delegator_id,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&DelegationRemovedMetadata {
                            delegator_id: *delegator_id,
                        }),
                    ),
                    DelegationEvent::DelegationStakeIncreased {
                        delegator_id,
                        new_stake,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&DelegationStakeUpdatedMetadata {
                            delegator_id:   *delegator_id,
                            new_stake_uccd: *new_stake,
                            increased:      true,
                        }),
                    ),
                    DelegationEvent::DelegationStakeDecreased {
                        delegator_id,
                        new_stake,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&DelegationStakeUpdatedMetadata {
                            delegator_id:   *delegator_id,
                            new_stake_uccd: *new_stake,
                            increased:      false,
                        }),
                    ),
                    DelegationEvent::DelegationSetRestakeEarnings {
                        delegator_id,
                        restake_earnings,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&DelegationSetRestakeEarningsMetadata {
                            delegator_id:     *delegator_id,
                            restake_earnings: *restake_earnings,
                        }),
                    ),
                    DelegationEvent::DelegationSetDelegationTarget {
                        delegator_id,
                        delegation_target,
                    } => normal_account_transaction_operation(
                        i as i64,
                        details,
                        None,
                        Some(&DelegationSetDelegationTargetMetadata {
                            delegator_id:      *delegator_id,
                            delegation_target: match delegation_target {
                                DelegationTarget::Passive => "passive".to_string(),
                                DelegationTarget::Baker {
                                    baker_id,
                                } => format!("baker:{}", baker_id.id),
                            },
                        }),
                    ),
                })
                .collect(),
            None,
        ),
    }
}

fn operations_and_metadata_from_account_creation_details(
    details: &AccountCreationDetails,
) -> Vec<Operation> {
    vec![Operation {
        operation_identifier: Box::new(OperationIdentifier {
            index:         0,
            network_index: None,
        }),
        related_operations:   None,
        _type:                OPERATION_TYPE_ACCOUNT_CREATION.to_string(),
        status:               Some(OPERATION_STATUS_OK.to_string()),
        account:              Some(Box::new(AccountIdentifier {
            address:     details.address.to_string(),
            sub_account: None,
            metadata:    None,
        })),
        amount:               None,
        coin_change:          None,
        metadata:             Some(
            serde_json::to_value(AccountCreatedMetadata {
                credential_type: match details.credential_type {
                    CredentialType::Initial => "initial".to_string(),
                    CredentialType::Normal => "normal".to_string(),
                },
                address:         details.address,
                registration_id: details.reg_id,
            })
            .unwrap(),
        ),
    }]
}

fn operations_and_metadata_from_chain_update_details(details: &UpdateDetails) -> Vec<Operation> {
    vec![Operation {
        operation_identifier: Box::new(OperationIdentifier {
            index:         0,
            network_index: None,
        }),
        related_operations:   None,
        _type:                OPERATION_TYPE_CHAIN_UPDATE.to_string(),
        status:               Some(OPERATION_STATUS_OK.to_string()),
        account:              None,
        amount:               None,
        coin_change:          None,
        metadata:             Some(
            serde_json::to_value(&ChainUpdateMetadata {
                effective_time: details.effective_time,
                payload:        details.payload.clone(),
            })
            .unwrap(),
        ),
    }]
}

fn contract_update_operations(
    details: &AccountTransactionDetails,
    effects: &[ContractTraceElement],
) -> Vec<Operation> {
    let mut ops = vec![];
    let mut next_index = 0;
    for e in effects.iter() {
        match e {
            ContractTraceElement::Updated {
                data,
            } => {
                // TODO Adapt and use 'simple_transfer_operations'.
                ops.push(account_transaction_operation::<Value>(
                    next_index,
                    details,
                    match &data.instigator {
                        Address::Account(a) => a.to_string(),
                        Address::Contract(a) => contract_address_string(a),
                    },
                    Some(amount_from_uccd(-(data.amount.micro_ccd() as i128))),
                    None,
                ));
                ops.push(account_transaction_operation::<Value>(
                    next_index + 1,
                    details,
                    contract_address_string(&data.address),
                    Some(amount_from_uccd(data.amount.micro_ccd() as i128)),
                    None,
                ));
                next_index += 2;
            }
            ContractTraceElement::Transferred {
                from,
                amount,
                to,
            } => {
                // TODO Adapt and use 'simple_transfer_operations'.
                ops.push(account_transaction_operation::<Value>(
                    next_index,
                    details,
                    contract_address_string(from),
                    Some(amount_from_uccd(-(amount.micro_ccd() as i128))),
                    None,
                ));
                ops.push(account_transaction_operation::<Value>(
                    next_index + 1,
                    details,
                    to.to_string(),
                    Some(amount_from_uccd(amount.micro_ccd() as i128)),
                    None,
                ));
                next_index += 2;
            }
            ContractTraceElement::Interrupted {
                ..
            } => {}
            ContractTraceElement::Resumed {
                ..
            } => {}
            ContractTraceElement::Upgraded {
                ..
            } => {}
        }
    }
    ops
}

fn simple_transfer_operations(
    details: &AccountTransactionDetails,
    amount: &Amount,
    to: &AccountAddress,
) -> Vec<Operation> {
    let sender_operation = account_transaction_operation::<Value>(
        0,
        details,
        details.sender.to_string(),
        Some(amount_from_uccd(-(amount.micro_ccd() as i128))),
        None,
    );
    let mut receiver_operation = account_transaction_operation::<Value>(
        1,
        details,
        to.to_string(),
        Some(amount_from_uccd(amount.micro_ccd() as i128)),
        None,
    );
    receiver_operation.related_operations =
        Some(vec![sender_operation.operation_identifier.deref().clone()]);
    vec![sender_operation, receiver_operation]
}

fn encrypted_transfer_operations(
    details: &AccountTransactionDetails,
    removed: &EncryptedAmountRemovedEvent,
    added: &NewEncryptedAmountEvent,
) -> Vec<Operation> {
    let sender_operation = account_transaction_operation(
        0,
        details,
        details.sender.to_string(), // assuming this to be the same as 'removed.account'
        None,
        Some(&EncryptedAmountTransferredSenderMetadata {
            new_encrypted_balance: removed.new_amount.clone(),
            encrypted_amount:      removed.input_amount.clone(),
            up_to_index:           removed.up_to_index,
        }),
    );
    let mut receiver_operation = account_transaction_operation(
        1,
        details,
        added.receiver.to_string(),
        None,
        Some(&EncryptedAmountTransferredReceiverMetadata {
            new_index:        added.new_index,
            encrypted_amount: added.encrypted_amount.clone(),
        }),
    );
    receiver_operation.related_operations =
        Some(vec![sender_operation.operation_identifier.deref().clone()]);
    vec![sender_operation, receiver_operation]
}

fn normal_account_transaction_operation<T: SerdeSerialize>(
    index: i64,
    details: &AccountTransactionDetails,
    amount: Option<rosetta::models::Amount>,
    metadata: Option<&T>,
) -> Operation {
    let account_address = details.sender.to_string();
    account_transaction_operation(index, details, account_address, amount, metadata)
}

fn account_transaction_operation<T: SerdeSerialize>(
    index: i64,
    details: &AccountTransactionDetails,
    account_address: String,
    amount: Option<rosetta::models::Amount>,
    metadata: Option<&T>,
) -> Operation {
    Operation {
        operation_identifier: Box::new(OperationIdentifier::new(index)),
        related_operations:   None,
        _type:                transaction_type_to_operation_type(details.transaction_type()),
        status:               Some(OPERATION_STATUS_OK.to_string()),
        account:              Some(Box::new(AccountIdentifier::new(account_address))),
        amount:               amount.map(Box::new),
        coin_change:          None,
        metadata:             metadata.map(serde_json::to_value).map(Result::unwrap),
    }
}

pub fn transaction_type_from_operation_type(type_: &str) -> ApiResult<TransactionType> {
    match type_ {
        #[allow(deprecated)]
        OPERATION_TYPE_ADD_BAKER => Ok(TransactionType::AddBaker),
        OPERATION_TYPE_DEPLOY_MODULE => Ok(TransactionType::DeployModule),
        #[allow(deprecated)]
        OPERATION_TYPE_ENCRYPTED_AMOUNT_TRANSFER => Ok(TransactionType::EncryptedAmountTransfer),
        OPERATION_TYPE_INIT_CONTRACT => Ok(TransactionType::InitContract),
        OPERATION_TYPE_REGISTER_DATA => Ok(TransactionType::RegisterData),
        #[allow(deprecated)]
        OPERATION_TYPE_REMOVE_BAKER => Ok(TransactionType::RemoveBaker),
        OPERATION_TYPE_TRANSFER => Ok(TransactionType::Transfer),
        #[allow(deprecated)]
        OPERATION_TYPE_TRANSFER_TO_ENCRYPTED => Ok(TransactionType::TransferToEncrypted),
        OPERATION_TYPE_TRANSFER_TO_PUBLIC => Ok(TransactionType::TransferToPublic),
        OPERATION_TYPE_TRANSFER_WITH_SCHEDULE => Ok(TransactionType::TransferWithSchedule),
        #[allow(deprecated)]
        OPERATION_TYPE_UPDATE_BAKER_KEYS => Ok(TransactionType::UpdateBakerKeys),
        OPERATION_TYPE_UPDATE_BAKER_RESTAKE_EARNINGS =>
        {
            #[allow(deprecated)]
            Ok(TransactionType::UpdateBakerRestakeEarnings)
        }
        #[allow(deprecated)]
        OPERATION_TYPE_UPDATE_BAKER_STAKE => Ok(TransactionType::UpdateBakerStake),
        OPERATION_TYPE_UPDATE_CONTRACT => Ok(TransactionType::Update),
        OPERATION_TYPE_UPDATE_CREDENTIAL_KEYS => Ok(TransactionType::UpdateCredentialKeys),
        OPERATION_TYPE_UPDATE_CREDENTIALS => Ok(TransactionType::UpdateCredentials),
        _ => Err(ApiError::UnsupportedOperationType(type_.to_string())),
    }
}

pub fn transaction_type_to_operation_type(type_: Option<TransactionType>) -> String {
    let res = match type_ {
        None => OPERATION_TYPE_UNKNOWN,
        Some(t) => match t {
            #[allow(deprecated)]
            TransactionType::AddBaker => OPERATION_TYPE_ADD_BAKER,
            TransactionType::DeployModule => OPERATION_TYPE_DEPLOY_MODULE,
            #[allow(deprecated)]
            TransactionType::EncryptedAmountTransfer => OPERATION_TYPE_ENCRYPTED_AMOUNT_TRANSFER,
            #[allow(deprecated)]
            TransactionType::EncryptedAmountTransferWithMemo => OPERATION_TYPE_TRANSFER,
            TransactionType::InitContract => OPERATION_TYPE_INIT_CONTRACT,
            TransactionType::RegisterData => OPERATION_TYPE_REGISTER_DATA,
            #[allow(deprecated)]
            TransactionType::RemoveBaker => OPERATION_TYPE_REMOVE_BAKER,
            TransactionType::Transfer => OPERATION_TYPE_TRANSFER,
            #[allow(deprecated)]
            TransactionType::TransferToEncrypted => OPERATION_TYPE_TRANSFER_TO_ENCRYPTED,
            TransactionType::TransferToPublic => OPERATION_TYPE_TRANSFER_TO_PUBLIC,
            TransactionType::TransferWithMemo => OPERATION_TYPE_TRANSFER,
            TransactionType::TransferWithScheduleAndMemo => OPERATION_TYPE_TRANSFER,
            TransactionType::TransferWithSchedule => OPERATION_TYPE_TRANSFER_WITH_SCHEDULE,
            TransactionType::Update => OPERATION_TYPE_UPDATE_CONTRACT,
            #[allow(deprecated)]
            TransactionType::UpdateBakerKeys => OPERATION_TYPE_UPDATE_BAKER_KEYS,
            #[allow(deprecated)]
            TransactionType::UpdateBakerRestakeEarnings => {
                OPERATION_TYPE_UPDATE_BAKER_RESTAKE_EARNINGS
            }
            #[allow(deprecated)]
            TransactionType::UpdateBakerStake => OPERATION_TYPE_UPDATE_BAKER_STAKE,
            TransactionType::UpdateCredentialKeys => OPERATION_TYPE_UPDATE_CREDENTIAL_KEYS,
            TransactionType::UpdateCredentials => OPERATION_TYPE_UPDATE_CREDENTIALS,
            TransactionType::ConfigureBaker => OPERATION_TYPE_CONFIGURE_BAKER,
            TransactionType::ConfigureDelegation => OPERATION_TYPE_CONFIGURE_DELEGATION,
        },
    };
    res.to_string()
}
