use crate::api::amount::amount_from_uccd;
use concordium_rust_sdk::common::types::{Amount, Timestamp, TransactionTime};
use concordium_rust_sdk::common::SerdeSerialize;
use concordium_rust_sdk::constants::EncryptedAmountsCurve;
use concordium_rust_sdk::encrypted_transfers::types::*;
use concordium_rust_sdk::id::types::AccountAddress;
use concordium_rust_sdk::types::*;
use rosetta::models::{
    AccountIdentifier, Operation, OperationIdentifier, Transaction, TransactionIdentifier,
};
use serde_json::{json, Error, Value};
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
    address: ContractAddress,
    amount: Amount,
    init_name: smart_contracts::InitName,
    events: Vec<smart_contracts::ContractEvent>,
}

#[derive(SerdeSerialize)]
struct ContractUpdateIssuedMetadata {
    // TODO Include 'effects'.
}

#[derive(SerdeSerialize)]
// TODO Name "transferred" for consistency?
struct AccountTransferMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    transferred_amount_uccd: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memo: Option<Memo>,
}

#[derive(SerdeSerialize)]
struct BakerAddedMetadata {
    baker_id: BakerId,
    account: AccountAddress,
    sign_key: BakerSignatureVerifyKey,
    election_key: BakerElectionVerifyKey,
    aggregation_key: BakerAggregationVerifyKey,
    stake_uccd: concordium_rust_sdk::common::types::Amount,
    restake_earnings: bool,
}

#[derive(SerdeSerialize)]
struct BakerRemovedMetadata {
    baker_id: BakerId,
}

#[derive(SerdeSerialize)]
struct BakerStakeUpdatedMetadata {
    baker_id: BakerId,
    new_stake_uccd: Amount,
    increased: bool,
}

#[derive(SerdeSerialize)]
struct BakerRestakeEarningsUpdatedMetadata {
    baker_id: BakerId,
    restake_earnings: bool,
}

#[derive(SerdeSerialize)]
struct BakerKeysUpdatedMetadata {
    baker_id: BakerId,
    account: AccountAddress,
    sign_key: BakerSignatureVerifyKey,
    election_key: BakerElectionVerifyKey,
    aggregation_key: BakerAggregationVerifyKey,
}

#[derive(SerdeSerialize)]
struct EncryptedAmountTransferredSenderMetadata {
    new_encrypted_balance: EncryptedAmount<EncryptedAmountsCurve>,
    encrypted_amount: EncryptedAmount<EncryptedAmountsCurve>,
    up_to_index: EncryptedAmountAggIndex,
}
#[derive(SerdeSerialize)]
struct EncryptedAmountTransferredReceiverMetadata {
    new_index: EncryptedAmountIndex,
    encrypted_amount: EncryptedAmount<EncryptedAmountsCurve>,
}

#[derive(SerdeSerialize)]
struct TransferredToEncryptedMetadata {
    amount: Amount,
    new_encrypted_amount: EncryptedAmount<EncryptedAmountsCurve>,
}

#[derive(SerdeSerialize)]
struct TransferredToPublicMetadata {
    address: AccountAddress,
    amount: Amount,
    new_encrypted_amount: EncryptedAmount<EncryptedAmountsCurve>,
    encrypted_amount: EncryptedAmount<EncryptedAmountsCurve>,
    up_to_index: EncryptedAmountAggIndex,
}

#[derive(SerdeSerialize)]
struct TransferredWithScheduleMetadata {
    receiver_address: AccountAddress,
    amounts: Vec<(Timestamp, Amount)>, // TODO convert to map
    #[serde(skip_serializing_if = "Option::is_none")]
    memo: Option<Memo>,
}

#[derive(SerdeSerialize)]
struct CredentialKeysUpdatedMetadata {
    credential_id: CredentialRegistrationID,
}

#[derive(SerdeSerialize)]
struct CredentialsUpdatedMetadata {
    removed_credential_ids: Vec<CredentialRegistrationID>,
    added_credential_ids: Vec<CredentialRegistrationID>,
    new_threshold: AccountThreshold,
}

#[derive(SerdeSerialize)]
struct DataRegisteredMetadata {
    data: RegisteredData,
}

#[derive(SerdeSerialize)]
struct AccountCreatedMetadata {
    credential_type: String,
    address: AccountAddress,
    registration_id: CredentialRegistrationID,
}

#[derive(SerdeSerialize)]
struct ChainUpdateMetadata {
    effective_time: TransactionTime,
    payload: UpdatePayload,
}

// pub fn map_transactions(transactions: Vec<BlockItemSummary>) -> Vec<Transaction> {
//     let mut result = Vec::with_capacity(transactions.len());
//     for transaction in transactions {
//         let t = map_transaction(&transaction);
//         result.push(t.clone());
//     }
//     result
// }

pub fn map_transaction(info: &BlockItemSummary) -> Transaction {
    let ((operations, extra_metadata), cost_uccd) = match &info.details {
        BlockItemSummaryDetails::AccountTransaction(details) => (
            operations_and_metadata_from_account_transaction_details(details),
            Some(details.cost),
        ),
        BlockItemSummaryDetails::AccountCreation(details) => (
            operations_and_metadata_from_account_creation_details(details),
            None,
        ),
        BlockItemSummaryDetails::Update(details) => (
            operations_and_metadata_from_chain_update_details(details),
            None,
        ),
    };
    let metadata = cost_uccd.map(|c| json!({ "cost_uccd": c }));
    Transaction {
        transaction_identifier: Box::new(TransactionIdentifier {
            hash: info.hash.to_string(),
        }),
        operations,
        related_transactions: None, // TODO
        metadata: transaction_metadata(metadata, extra_metadata.map(Result::unwrap)),
    }
}

fn transaction_metadata(left: Option<Value>, right: Option<Value>) -> Option<Value> {
    merge_optional_values(left, right)
}

fn merge_optional_values(left: Option<Value>, right: Option<Value>) -> Option<Value> {
    match left.clone() {
        None => right,
        Some(l) => match right.clone() {
            None => left.clone(),
            Some(r) => Some(merge_values(l, r)),
        },
    }
}

fn merge_values(left: Value, right: Value) -> Value {
    match (left.clone().as_object_mut(), right.clone().as_object()) {
        (Some(l), Some(r)) => {
            l.extend(r.clone());
            serde_json::to_value(l).unwrap()
        }
        (Some(_), None) => left,
        (None, Some(_)) => right,
        (None, None) => left,
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
                operation_identifier: Box::new(OperationIdentifier {
                    index: 0,
                    network_index: None,
                }),
                related_operations: None,
                _type: transaction_type_to_string(*transaction_type),
                status: Some("fail".to_string()),
                account: Some(Box::new(AccountIdentifier {
                    address: details.sender.to_string(),
                    sub_account: None,
                    metadata: None,
                })),
                amount: Some(Box::new(amount_from_uccd(details.cost.microgtu as i64))),
                coin_change: None,
                metadata: Some(
                    serde_json::to_value(&TransactionRejectedMetadata {
                        reject_reason: reject_reason.clone(),
                    })
                    .unwrap(),
                ),
            }],
            None,
        ),
        AccountTransactionEffects::ModuleDeployed { module_ref } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&ModuleDeployedMetadata {
                    module_ref: *module_ref,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::ContractInitialized { data } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&ContractInitializedMetadata {
                    module_ref: data.origin_ref,
                    address: data.address,
                    amount: data.amount,
                    init_name: data.init_name.clone(),
                    events: data.events.clone(),
                }),
            )],
            None,
        ),
        AccountTransactionEffects::ContractUpdateIssued { .. } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&ContractUpdateIssuedMetadata {}),
            )],
            None,
        ),
        AccountTransactionEffects::AccountTransfer { amount, to } => (
            simple_transfer_operations(details, amount, to),
            Some(serde_json::to_value(&AccountTransferMetadata {
                transferred_amount_uccd: Some(*amount),
                memo: None,
            })),
        ),
        AccountTransactionEffects::AccountTransferWithMemo { amount, to, memo } => (
            simple_transfer_operations(details, amount, to),
            Some(serde_json::to_value(&AccountTransferMetadata {
                transferred_amount_uccd: Some(*amount),
                memo: Some(memo.clone()),
            })),
        ),
        AccountTransactionEffects::BakerAdded { data } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&BakerAddedMetadata {
                    baker_id: data.keys_event.baker_id,
                    account: data.keys_event.account,
                    sign_key: data.keys_event.sign_key.clone(),
                    election_key: data.keys_event.election_key.clone(),
                    aggregation_key: data.keys_event.aggregation_key.clone(),
                    stake_uccd: data.stake,
                    restake_earnings: data.restake_earnings,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::BakerRemoved { baker_id } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&BakerRemovedMetadata {
                    baker_id: *baker_id,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::BakerStakeUpdated {
            baker_id,
            new_stake,
            increased,
        } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&BakerStakeUpdatedMetadata {
                    baker_id: *baker_id,
                    new_stake_uccd: *new_stake,
                    increased: *increased,
                }),
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
                Some(&BakerRestakeEarningsUpdatedMetadata {
                    baker_id: *baker_id,
                    restake_earnings: *restake_earnings,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::BakerKeysUpdated { data } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&BakerKeysUpdatedMetadata {
                    baker_id: data.baker_id,
                    account: data.account,
                    sign_key: data.sign_key.clone(),
                    election_key: data.election_key.clone(),
                    aggregation_key: data.aggregation_key.clone(),
                }),
            )],
            None,
        ),
        AccountTransactionEffects::EncryptedAmountTransferred { removed, added } => (
            encrypted_transfer_operations(details, removed, added),
            Some(serde_json::to_value(&AccountTransferMetadata {
                transferred_amount_uccd: None,
                memo: None,
            })),
        ),
        AccountTransactionEffects::EncryptedAmountTransferredWithMemo {
            removed,
            added,
            memo,
        } => (
            encrypted_transfer_operations(details, removed, added),
            Some(serde_json::to_value(&AccountTransferMetadata {
                transferred_amount_uccd: None,
                memo: Some(memo.clone()),
            })),
        ),
        AccountTransactionEffects::TransferredToEncrypted { data } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&TransferredToEncryptedMetadata {
                    amount: data.amount,
                    new_encrypted_amount: data.new_amount.clone(),
                }),
            )],
            None,
        ),
        AccountTransactionEffects::TransferredToPublic { removed, amount } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&TransferredToPublicMetadata {
                    address: removed.account,
                    amount: *amount,
                    new_encrypted_amount: removed.new_amount.clone(),
                    encrypted_amount: removed.input_amount.clone(),
                    up_to_index: removed.up_to_index,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::TransferredWithSchedule { to, amount } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&TransferredWithScheduleMetadata {
                    receiver_address: *to,
                    amounts: amount.clone(),
                    memo: None,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::TransferredWithScheduleAndMemo { to, amount, memo } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&TransferredWithScheduleMetadata {
                    receiver_address: *to,
                    amounts: amount.clone(),
                    memo: Some(memo.clone()),
                }),
            )],
            None,
        ),
        AccountTransactionEffects::CredentialKeysUpdated { cred_id } => (
            vec![self::normal_account_transaction_operation(
                0,
                details,
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
                Some(&CredentialsUpdatedMetadata {
                    removed_credential_ids: removed_cred_ids.clone(),
                    added_credential_ids: new_cred_ids.clone(),
                    new_threshold: *new_threshold,
                }),
            )],
            None,
        ),
        AccountTransactionEffects::DataRegistered { data } => (
            vec![normal_account_transaction_operation(
                0,
                details,
                Some(&DataRegisteredMetadata { data: data.clone() }),
            )],
            None,
        ),
    }
}

fn operations_and_metadata_from_account_creation_details(
    details: &AccountCreationDetails,
) -> (Vec<Operation>, Option<Result<Value, Error>>) {
    (
        vec![Operation {
            operation_identifier: Box::new(OperationIdentifier {
                index: 0,
                network_index: None,
            }),
            related_operations: None,
            _type: "AccountCreation".to_string(),
            status: Some("ok".to_string()),
            account: Some(Box::new(AccountIdentifier {
                address: details.address.to_string(),
                sub_account: None,
                metadata: None,
            })),
            amount: None,
            coin_change: None,
            metadata: Some(
                serde_json::to_value(&AccountCreatedMetadata {
                    credential_type: match details.credential_type {
                        CredentialType::Initial => "initial".to_string(),
                        CredentialType::Normal => "normal".to_string(),
                    },
                    address: details.address,
                    registration_id: details.reg_id.clone(),
                })
                .unwrap(),
            ),
        }],
        None,
    )
}

fn operations_and_metadata_from_chain_update_details(
    details: &UpdateDetails,
) -> (Vec<Operation>, Option<Result<Value, Error>>) {
    (
        vec![Operation {
            operation_identifier: Box::new(OperationIdentifier {
                index: 0,
                network_index: None,
            }),
            related_operations: None,
            _type: "ChainUpdate".to_string(),
            status: Some("ok".to_string()),
            account: None,
            amount: None,
            coin_change: None,
            metadata: Some(
                serde_json::to_value(&ChainUpdateMetadata {
                    effective_time: details.effective_time,
                    payload: details.payload.clone(),
                })
                .unwrap(),
            ),
        }],
        None,
    )
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
        Some(amount_from_uccd(
            -((amount.microgtu + details.cost.microgtu) as i64),
        )),
        None,
    );
    let mut receiver_operation = account_transaction_operation::<Value>(
        1,
        details,
        to.to_string(),
        Some(amount_from_uccd(amount.microgtu as i64)),
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
            encrypted_amount: removed.input_amount.clone(),
            up_to_index: removed.up_to_index,
        }),
    );
    let mut receiver_operation = account_transaction_operation(
        1,
        details,
        added.receiver.to_string(),
        None,
        Some(&EncryptedAmountTransferredReceiverMetadata {
            new_index: added.new_index,
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
    metadata: Option<&T>,
) -> Operation {
    let account_address = details.sender.to_string();
    let amount = amount_from_uccd(details.cost.microgtu as i64);
    account_transaction_operation(index, details, account_address, Some(amount), metadata)
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
        related_operations: None,
        _type: transaction_type_to_string(details.transaction_type()),
        status: Some("ok".to_string()),
        account: Some(Box::new(AccountIdentifier {
            address: account_address,
            sub_account: None,
            metadata: None,
        })),
        amount: amount.map(Box::new),
        coin_change: None,
        metadata: metadata.map(serde_json::to_value).map(Result::unwrap),
    }
}

fn transaction_type_to_string(type_: Option<TransactionType>) -> String {
    let res = match type_ {
        None => "unknown",
        Some(t) => match t {
            TransactionType::DeployModule => "deploy_module",
            TransactionType::InitContract => "init_contract",
            TransactionType::Update => "update_contract",
            TransactionType::Transfer => "transfer",
            TransactionType::AddBaker => "add_baker",
            TransactionType::RemoveBaker => "remove_baker",
            TransactionType::UpdateBakerStake => "update_baker_stake",
            TransactionType::UpdateBakerRestakeEarnings => "update_baker_restake_earnings",
            TransactionType::UpdateBakerKeys => "update_baker_keys",
            TransactionType::UpdateCredentialKeys => "update_credential_keys",
            TransactionType::EncryptedAmountTransfer => "encrypted_amount_transfer",
            TransactionType::TransferToEncrypted => "transfer_to_encrypted",
            TransactionType::TransferToPublic => "transfer_to_public",
            TransactionType::TransferWithSchedule => "transfer_with_schedule",
            TransactionType::UpdateCredentials => "update_credentials",
            TransactionType::RegisterData => "register_data",
            TransactionType::TransferWithMemo => "transfer_with_memo",
            TransactionType::EncryptedAmountTransferWithMemo => {
                "encrypted_amount_transfer_with_memo"
            }
            TransactionType::TransferWithScheduleAndMemo => "transfer_with_schedule_and_memo",
        },
    };
    res.to_string()
}
