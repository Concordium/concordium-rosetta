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
    module_ref: smart_contracts::ModuleRef,
}

#[derive(SerdeSerialize)]
struct ContractInitializedMetadata {
    module_ref: smart_contracts::ModuleRef,
    address:    ContractAddress,
    init_name:  smart_contracts::InitName,
    events:     Vec<smart_contracts::ContractEvent>,
}

#[derive(SerdeSerialize)]
struct ContractUpdateIssuedMetadata {
    // TODO Include 'effects'.
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
pub const ACCOUNT_CONTRACT_PREFIX: &str = "contract:";

pub const OPERATION_STATUS_OK: &str = "ok";
pub const OPERATION_STATUS_FAIL: &str = "fail";

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

pub const TRANSACTION_HASH_TOKENOMICS: &str = "tokenomics";

pub fn map_transaction(info: &BlockItemSummary) -> Transaction {
    let (operations, extra_metadata) = match &info.details {
        BlockItemSummaryDetails::AccountTransaction(details) => {
            let (ops, metadata) = operations_and_metadata_from_account_transaction_details(details);
            let mut ops_with_fee = ops.clone();
            if details.cost.microgtu != 0 {
                ops_with_fee.push(Operation {
                    operation_identifier: Box::new(OperationIdentifier::new(ops.len() as i64)),
                    related_operations:   None,
                    _type:                OPERATION_TYPE_FEE.to_string(),
                    status:               Some(OPERATION_STATUS_OK.to_string()),
                    account:              Some(Box::new(AccountIdentifier::new(
                        details.sender.to_string(),
                    ))),
                    amount:               Some(Box::new(amount_from_uccd(
                        -(details.cost.microgtu as i128),
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
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(amount_from_uccd(data.amount.microgtu as i128)),
                Some(&ContractInitializedMetadata {
                    module_ref: data.origin_ref,
                    address:    data.address,
                    init_name:  data.init_name.clone(),
                    events:     data.events.clone(),
                }),
            )],
            None,
        ),
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
            Some(serde_json::to_value(&MemoMetadata {
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
            Some(serde_json::to_value(&MemoMetadata {
                memo: Some(memo.clone()),
            })),
        ),
        AccountTransactionEffects::TransferredToEncrypted {
            data,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(amount_from_uccd(-(data.amount.microgtu as i128))),
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
                Some(amount_from_uccd(amount.microgtu as i128)),
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
                &Amount::from(amount.iter().map(|(_, a)| a.microgtu).sum::<u64>()),
                to,
            ),
            Some(serde_json::to_value(&TransferredWithScheduleMetadata {
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
                &Amount::from(amount.iter().map(|(_, a)| a.microgtu).sum::<u64>()),
                to,
            ),
            Some(serde_json::to_value(&TransferredWithScheduleMetadata {
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
                    credential_id: cred_id.clone(),
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
            serde_json::to_value(&AccountCreatedMetadata {
                credential_type: match details.credential_type {
                    CredentialType::Initial => "initial".to_string(),
                    CredentialType::Normal => "normal".to_string(),
                },
                address:         details.address,
                registration_id: details.reg_id.clone(),
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
    let mut updated_amount = 0;
    let mut ops = vec![];
    let mut next_index = 1;
    for e in effects.iter() {
        match e {
            ContractTraceElement::Updated {
                data,
            } => updated_amount += data.amount.microgtu as i128,
            ContractTraceElement::Transferred {
                from,
                amount,
                to,
            } => {
                ops.push(account_transaction_operation::<Value>(
                    next_index,
                    details,
                    format!(
                        "{}{}_{}",
                        ACCOUNT_CONTRACT_PREFIX, from.index.index, from.subindex.sub_index
                    ),
                    Some(amount_from_uccd(-(amount.microgtu as i128))),
                    None,
                ));
                ops.push(account_transaction_operation::<Value>(
                    next_index + 1,
                    details,
                    to.to_string(),
                    Some(amount_from_uccd(amount.microgtu as i128)),
                    None,
                ));
                next_index += 2;
            }
        }
    }
    let mut res = vec![normal_account_transaction_operation(
        0,
        details,
        Some(amount_from_uccd(-updated_amount)),
        Some(&ContractUpdateIssuedMetadata {}),
    )];
    res.extend(ops);
    res
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
        Some(amount_from_uccd(-(amount.microgtu as i128))),
        None,
    );
    let mut receiver_operation = account_transaction_operation::<Value>(
        1,
        details,
        to.to_string(),
        Some(amount_from_uccd(amount.microgtu as i128)),
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
        operation_identifier: Box::new(OperationIdentifier {
            index,
            network_index: None,
        }),
        related_operations:   None,
        _type:                transaction_type_to_operation_type(details.transaction_type()),
        status:               Some(OPERATION_STATUS_OK.to_string()),
        account:              Some(Box::new(AccountIdentifier {
            address:     account_address,
            sub_account: None,
            metadata:    None,
        })),
        amount:               amount.map(Box::new),
        coin_change:          None,
        metadata:             metadata.map(serde_json::to_value).map(Result::unwrap),
    }
}

pub fn transaction_type_from_operation_type(type_: &str) -> ApiResult<TransactionType> {
    match type_ {
        OPERATION_TYPE_ADD_BAKER => Ok(TransactionType::AddBaker),
        OPERATION_TYPE_DEPLOY_MODULE => Ok(TransactionType::DeployModule),
        OPERATION_TYPE_ENCRYPTED_AMOUNT_TRANSFER => Ok(TransactionType::EncryptedAmountTransfer),
        OPERATION_TYPE_INIT_CONTRACT => Ok(TransactionType::InitContract),
        OPERATION_TYPE_REGISTER_DATA => Ok(TransactionType::RegisterData),
        OPERATION_TYPE_REMOVE_BAKER => Ok(TransactionType::RemoveBaker),
        OPERATION_TYPE_TRANSFER => Ok(TransactionType::Transfer),
        OPERATION_TYPE_TRANSFER_TO_ENCRYPTED => Ok(TransactionType::TransferToEncrypted),
        OPERATION_TYPE_TRANSFER_TO_PUBLIC => Ok(TransactionType::TransferToPublic),
        OPERATION_TYPE_TRANSFER_WITH_SCHEDULE => Ok(TransactionType::TransferWithSchedule),
        OPERATION_TYPE_UPDATE_BAKER_KEYS => Ok(TransactionType::UpdateBakerKeys),
        OPERATION_TYPE_UPDATE_BAKER_RESTAKE_EARNINGS => {
            Ok(TransactionType::UpdateBakerRestakeEarnings)
        }
        OPERATION_TYPE_UPDATE_BAKER_STAKE => Ok(TransactionType::UpdateBakerStake),
        OPERATION_TYPE_UPDATE_CONTRACT => Ok(TransactionType::Update),
        OPERATION_TYPE_UPDATE_CREDENTIAL_KEYS => Ok(TransactionType::UpdateCredentialKeys),
        OPERATION_TYPE_UPDATE_CREDENTIALS => Ok(TransactionType::UpdateCredentials),
        _ => Err(ApiError::UnsupportedOperationType(type_.to_string())),
    }
}

pub fn transaction_type_to_operation_type(type_: Option<TransactionType>) -> String {
    let res = match type_ {
        None => "unknown",
        Some(t) => match t {
            TransactionType::AddBaker => OPERATION_TYPE_ADD_BAKER,
            TransactionType::DeployModule => OPERATION_TYPE_DEPLOY_MODULE,
            TransactionType::EncryptedAmountTransfer => OPERATION_TYPE_ENCRYPTED_AMOUNT_TRANSFER,
            TransactionType::EncryptedAmountTransferWithMemo => OPERATION_TYPE_TRANSFER,
            TransactionType::InitContract => OPERATION_TYPE_INIT_CONTRACT,
            TransactionType::RegisterData => OPERATION_TYPE_REGISTER_DATA,
            TransactionType::RemoveBaker => OPERATION_TYPE_REMOVE_BAKER,
            TransactionType::Transfer => OPERATION_TYPE_TRANSFER,
            TransactionType::TransferToEncrypted => OPERATION_TYPE_TRANSFER_TO_ENCRYPTED,
            TransactionType::TransferToPublic => OPERATION_TYPE_TRANSFER_TO_PUBLIC,
            TransactionType::TransferWithMemo => OPERATION_TYPE_TRANSFER,
            TransactionType::TransferWithScheduleAndMemo => OPERATION_TYPE_TRANSFER,
            TransactionType::TransferWithSchedule => OPERATION_TYPE_TRANSFER_WITH_SCHEDULE,
            TransactionType::Update => OPERATION_TYPE_UPDATE_CONTRACT,
            TransactionType::UpdateBakerKeys => OPERATION_TYPE_UPDATE_BAKER_KEYS,
            TransactionType::UpdateBakerRestakeEarnings => {
                OPERATION_TYPE_UPDATE_BAKER_RESTAKE_EARNINGS
            }
            TransactionType::UpdateBakerStake => OPERATION_TYPE_UPDATE_BAKER_STAKE,
            TransactionType::UpdateCredentialKeys => OPERATION_TYPE_UPDATE_CREDENTIAL_KEYS,
            TransactionType::UpdateCredentials => OPERATION_TYPE_UPDATE_CREDENTIALS,
        },
    };
    res.to_string()
}
