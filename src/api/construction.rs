use crate::{
    api::{
        amount::{amount_from_uccd, uccd_from_amount},
        error::{ApiError, ApiError::RequiredFieldMissing, ApiResult, InvalidSignatureError},
        query::account_address_from_identifier,
        transaction::{
            transaction_type_from_operation_type, transaction_type_to_operation_type, MemoMetadata,
            OPERATION_TYPE_TRANSFER,
        },
    },
    NetworkValidator, QueryHelper,
};
use concordium_rust_sdk::{
    common::{
        types::{CredentialIndex, KeyIndex, TransactionSignature, TransactionTime},
        SerdeDeserialize, SerdeSerialize,
    },
    constants::DEFAULT_NETWORK_ID,
    id::types::AccountAddress,
    types::{
        transactions::{
            compute_transaction_sign_hash, construct, construct::GivenEnergy, cost,
            AccountTransaction, BlockItem, EncodedPayload, Payload, PayloadLike, TransactionHeader,
        },
        Memo, Nonce, TransactionType,
    },
};
use rosetta::models::*;
use std::{collections::BTreeMap, ops::Deref, str::FromStr};

#[derive(Clone)]
pub struct ConstructionApi {
    network_validator: NetworkValidator,
    query_helper:      QueryHelper,
}

#[derive(SerdeSerialize, SerdeDeserialize)]
struct ConstructionOptions {
    sender: AccountAddress,
}

#[derive(SerdeSerialize)]
struct MetadataResponseMetadata {
    account_nonce: Nonce,
}

#[derive(SerdeDeserialize)]
struct PayloadRequestMetadata {
    account_nonce:      Nonce,
    signature_count:    u32,
    expiry_unix_millis: u64, // using milliseconds for consistency with block timestamp field
    memo:               Option<Memo>,
}

struct ParsedTransferOperation {
    account_address: AccountAddress,
    amount_uccd:     i64,
}

enum ParsedOperation {
    Transfer(ParsedTransferOperation),
}

struct ParsedTransferTransaction {
    sender_address:   AccountAddress,
    receiver_address: AccountAddress,
    amount_uccd:      u64,
}

enum ParsedTransaction {
    Transfer(ParsedTransferTransaction),
}

// TODO Seems redundant - should just use 'AccountTransaction<EncodedPayload>'
// with empty signature set..?
#[derive(SerdeSerialize, SerdeDeserialize)]
struct UnsignedTransaction {
    header:  TransactionHeader,
    payload: EncodedPayload,
}

impl ConstructionApi {
    pub fn new(network_validator: NetworkValidator, query_helper: QueryHelper) -> Self {
        Self {
            network_validator,
            query_helper,
        }
    }

    pub async fn preprocess(
        &self,
        req: ConstructionPreprocessRequest,
    ) -> ApiResult<ConstructionPreprocessResponse> {
        self.network_validator.validate_network_identifier(*req.network_identifier)?;
        if req.max_fee.is_some() {
            // TODO can query field name from serde?
            return Err(ApiError::UnsupportedFieldPresent("max_fee".to_string()));
        }
        if req.suggested_fee_multiplier.is_some() {
            return Err(ApiError::UnsupportedFieldPresent("suggested_fee_multiplier".to_string()));
        }
        let options = match transaction_from_operations(&req.operations)? {
            ParsedTransaction::Transfer(transfer_tx) => ConstructionOptions {
                sender: transfer_tx.sender_address,
            },
        };
        Ok(ConstructionPreprocessResponse {
            options:              Some(
                serde_json::to_value(&options)
                    .map_err(|err| ApiError::JsonEncodingFailed("options".to_string(), err))?,
            ),
            required_public_keys: Some(vec![AccountIdentifier::new(options.sender.to_string())]),
        })
    }

    pub async fn metadata(
        &self,
        req: ConstructionMetadataRequest,
    ) -> ApiResult<ConstructionMetadataResponse> {
        self.network_validator.validate_network_identifier(*req.network_identifier)?;
        if req.public_keys.is_some() {
            return Err(ApiError::UnsupportedFieldPresent("public_keys".to_string()));
        }
        let opts = match req.options {
            None => return Err(ApiError::RequiredFieldMissing("options".to_string())),
            Some(v) => serde_json::from_value::<ConstructionOptions>(v)
                .map_err(|_| ApiError::InvalidConstructionOptions)?,
        };
        let consensus_status = self.query_helper.client.clone().get_consensus_status().await?;
        let sender_info = self
            .query_helper
            .client
            .clone()
            .get_account_info(opts.sender, &consensus_status.last_finalized_block)
            .await?;
        // TODO Should include account's credential keys? Would enable signature
        // verification later on.
        Ok(ConstructionMetadataResponse {
            metadata:      serde_json::to_value(MetadataResponseMetadata {
                account_nonce: sender_info.account_nonce,
            })
            .unwrap(),
            suggested_fee: None,
        })
    }

    pub async fn payloads(
        &self,
        req: ConstructionPayloadsRequest,
    ) -> ApiResult<ConstructionPayloadsResponse> {
        self.network_validator.validate_network_identifier(*req.network_identifier)?;
        if req.public_keys.is_some() {
            return Err(ApiError::UnsupportedFieldPresent("public_keys".to_string()));
        }
        let metadata = match req.metadata {
            None => return Err(RequiredFieldMissing("metadata".to_string())),
            Some(v) => serde_json::from_value::<PayloadRequestMetadata>(v)
                .map_err(|_| ApiError::InvalidPayloadsMetadata)?,
        };
        let parsed_transaction = transaction_from_operations(&req.operations)?;
        let (builder, account_address) = match parsed_transaction {
            ParsedTransaction::Transfer(tx) => {
                let to_address = tx.receiver_address;
                let amount = tx.amount_uccd.into();
                let payload = match metadata.memo {
                    None => Payload::Transfer {
                        to_address,
                        amount,
                    },
                    Some(memo) => Payload::TransferWithMemo {
                        to_address,
                        amount,
                        memo,
                    },
                };
                let pre_tx = construct::make_transaction(
                    tx.sender_address,
                    metadata.account_nonce,
                    TransactionTime::from_seconds(metadata.expiry_unix_millis / 1000),
                    GivenEnergy::Add {
                        num_sigs: metadata.signature_count,
                        energy:   cost::SIMPLE_TRANSFER,
                    },
                    payload,
                );
                (pre_tx, tx.sender_address)
            }
        };
        Ok(ConstructionPayloadsResponse {
            unsigned_transaction: serde_json::to_string(&UnsignedTransaction {
                header:  builder.header.clone(),
                payload: builder.encoded.clone(),
            })
            .map_err(|err| ApiError::JsonEncodingFailed("unsigned_transaction".to_string(), err))?,
            payloads:             vec![SigningPayload {
                address:            None, // deprecated
                account_identifier: Some(Box::new(AccountIdentifier::new(
                    account_address.to_string(),
                ))),
                hex_bytes:          compute_transaction_sign_hash(
                    &builder.header,
                    &builder.encoded,
                )
                .to_string(),
                signature_type:     Some(SignatureType::Ed25519),
            }],
        })
    }

    pub async fn parse(
        &self,
        req: ConstructionParseRequest,
    ) -> ApiResult<ConstructionParseResponse> {
        self.network_validator.validate_network_identifier(*req.network_identifier)?;

        let (header, encoded_payload, signature) = if !req.signed {
            let unsigned_tx = decode_unsigned_transaction(req.transaction.as_str())?;
            (unsigned_tx.header, unsigned_tx.payload, None)
        } else {
            let signed_tx = decode_signed_transaction(req.transaction.as_str())?;
            (signed_tx.header, signed_tx.payload, Some(signed_tx.signature))
        };
        let payload = match encoded_payload.decode() {
            Err(_) => return Err(ApiError::InvalidEncodedPayload),
            Ok(p) => p,
        };

        let (operations, memo) = operations_from_transaction(&header, &payload)?;
        let metadata = memo.map(|m| {
            serde_json::to_value(&MemoMetadata {
                memo: Some(m),
            })
            .unwrap()
        });
        match signature {
            None => Ok(ConstructionParseResponse {
                operations,
                signers: None,
                account_identifier_signers: None,
                metadata,
            }),

            Some(_) => {
                Ok(ConstructionParseResponse {
                    operations,
                    signers: None, // deprecated
                    account_identifier_signers: Some(vec![AccountIdentifier::new(
                        header.sender.to_string(),
                    )]),
                    metadata,
                })
            }
        }
    }

    pub async fn combine(
        &self,
        req: ConstructionCombineRequest,
    ) -> ApiResult<ConstructionCombineResponse> {
        self.network_validator.validate_network_identifier(*req.network_identifier)?;
        let unsigned_tx = decode_unsigned_transaction(req.unsigned_transaction.as_str())?;
        let mut signatures: BTreeMap<
            CredentialIndex,
            BTreeMap<KeyIndex, concordium_rust_sdk::common::types::Signature>,
        > = BTreeMap::new();
        for s in req.signatures.iter() {
            let hex_bytes = &s.hex_bytes;
            let (idxs_str, sig_hex_bytes) = match hex_bytes.split_once('/') {
                None => {
                    return Err(ApiError::InvalidSignature(
                        hex_bytes.clone(),
                        InvalidSignatureError::MissingSeparator('/'.to_string()),
                    ))
                }
                Some(v) => v,
            };
            let (cred_idx_str, key_idx_str) = match idxs_str.split_once(':') {
                None => {
                    return Err(ApiError::InvalidSignature(
                        hex_bytes.clone(),
                        InvalidSignatureError::MissingIndexSeparator(':'.to_string()),
                    ))
                }
                Some(v) => v,
            };
            let cred_idx = CredentialIndex {
                index: match u8::from_str(cred_idx_str) {
                    Err(_) => {
                        return Err(ApiError::InvalidSignature(
                            hex_bytes.clone(),
                            InvalidSignatureError::InvalidCredentialIndex(cred_idx_str.to_string()),
                        ))
                    }
                    Ok(v) => v,
                },
            };
            let key_idx = KeyIndex(match u8::from_str(key_idx_str) {
                Err(_) => {
                    return Err(ApiError::InvalidSignature(
                        hex_bytes.clone(),
                        InvalidSignatureError::InvalidKeyIndex(key_idx_str.to_string()),
                    ))
                }
                Ok(v) => v,
            });
            let sig = match hex::decode(&sig_hex_bytes) {
                Err(_) => {
                    return Err(ApiError::InvalidSignature(
                        hex_bytes.clone(),
                        InvalidSignatureError::InvalidSignatureHexBytes(sig_hex_bytes.to_string()),
                    ))
                }
                Ok(v) => v,
            };

            let cred_signatures = signatures.entry(cred_idx).or_default();
            cred_signatures.insert(key_idx, concordium_rust_sdk::common::types::Signature {
                sig,
            });
        }

        let tx = serde_json::to_string(&AccountTransaction {
            signature: TransactionSignature {
                signatures,
            },
            header:    unsigned_tx.header,
            payload:   unsigned_tx.payload.encode(),
        })
        .map_err(|err| ApiError::JsonEncodingFailed("signed_transaction".to_string(), err))?;
        Ok(ConstructionCombineResponse {
            signed_transaction: tx,
        })
    }

    pub async fn submit(
        &self,
        req: ConstructionSubmitRequest,
    ) -> ApiResult<TransactionIdentifierResponse> {
        self.network_validator.validate_network_identifier(*req.network_identifier)?;

        let block_item = parse_block_item(req.signed_transaction.as_str())?;
        let success = self
            .query_helper
            .client
            .clone()
            .send_transaction(DEFAULT_NETWORK_ID, &block_item)
            .await?;
        if !success {
            // TODO Verify signatures in this case?
            return Err(ApiError::TransactionNotAccepted);
        }
        Ok(TransactionIdentifierResponse::new(TransactionIdentifier::new(
            block_item.hash().to_string(),
        )))
    }

    pub async fn hash(
        &self,
        req: ConstructionHashRequest,
    ) -> ApiResult<TransactionIdentifierResponse> {
        self.network_validator.validate_network_identifier(*req.network_identifier)?;
        let block_item = parse_block_item(req.signed_transaction.as_str())?;
        Ok(TransactionIdentifierResponse::new(TransactionIdentifier::new(
            block_item.hash().to_string(),
        )))
    }
}

fn parse_operation(op: &Operation) -> ApiResult<ParsedOperation> {
    match transaction_type_from_operation_type(op._type.as_str()) {
        Ok(TransactionType::Transfer) => {
            // Covers transfers with and without memo.
            let amount_uccd = match op.amount.as_deref() {
                Some(a) => uccd_from_amount(a.deref()),
                None => Err(ApiError::RequiredFieldMissing("amount".to_string())),
            }?;
            let account_address = match op.account.clone() {
                None => Err(ApiError::RequiredFieldMissing("account".to_string())),
                Some(a) => account_address_from_identifier(a.deref()),
            }?;
            Ok(ParsedOperation::Transfer(ParsedTransferOperation {
                account_address,
                amount_uccd,
            }))
        }
        _ => Err(ApiError::UnsupportedOperationType(op._type.clone())),
    }
}

fn parse_operations(ops: &[Operation]) -> ApiResult<Vec<ParsedOperation>> {
    ops.iter().map(parse_operation).collect()
}

fn parse_transaction(ops: &[ParsedOperation]) -> ApiResult<ParsedTransaction> {
    match ops {
        [ParsedOperation::Transfer(sender), ParsedOperation::Transfer(receiver)] => {
            parse_transfer_transaction(sender, receiver)
        }
        _ => Err(ApiError::InconsistentOperations(
            "invalid type or number of operations".to_string(),
        )),
    }
}

fn parse_transfer_transaction(
    sender: &ParsedTransferOperation,
    receiver: &ParsedTransferOperation,
) -> Result<ParsedTransaction, ApiError> {
    if sender.amount_uccd >= 0 {
        return Err(ApiError::InconsistentOperations(
            "amount in first transfer operation must be negative".to_string(),
        ));
    }
    if receiver.amount_uccd <= 0 {
        return Err(ApiError::InconsistentOperations(
            "amount in second transfer operation must be positive".to_string(),
        ));
    }
    if sender.amount_uccd != -receiver.amount_uccd {
        return Err(ApiError::InconsistentOperations(
            "amount in transfer operations must sum to zero".to_string(),
        ));
    }
    Ok(ParsedTransaction::Transfer(ParsedTransferTransaction {
        sender_address:   sender.account_address,
        receiver_address: receiver.account_address,
        amount_uccd:      receiver.amount_uccd as u64,
    }))
}

fn transaction_from_operations(ops: &[Operation]) -> ApiResult<ParsedTransaction> {
    parse_transaction(&parse_operations(ops)?)
}

fn parse_block_item(signed_transaction: &str) -> ApiResult<BlockItem<EncodedPayload>> {
    Ok(concordium_rust_sdk::types::transactions::BlockItem::AccountTransaction(
        decode_signed_transaction(signed_transaction)?,
    ))
}

fn decode_unsigned_transaction(unsigned_transaction: &str) -> ApiResult<UnsignedTransaction> {
    serde_json::from_str::<UnsignedTransaction>(unsigned_transaction)
        .map_err(|_| ApiError::InvalidUnsignedTransaction)
}

fn decode_signed_transaction(
    signed_transaction: &str,
) -> ApiResult<AccountTransaction<EncodedPayload>> {
    serde_json::from_str::<AccountTransaction<EncodedPayload>>(signed_transaction)
        .map_err(|_| ApiError::InvalidSignedTransaction)
}

fn operations_from_transaction(
    header: &TransactionHeader,
    payload: &Payload,
) -> ApiResult<(Vec<Operation>, Option<Memo>)> {
    match payload {
        Payload::Transfer {
            to_address,
            amount,
        } => operations_from_transfer_transaction(
            &header.sender,
            to_address,
            amount.microgtu as i64,
            None,
        ),
        Payload::TransferWithMemo {
            to_address,
            amount,
            memo,
        } => operations_from_transfer_transaction(
            &header.sender,
            to_address,
            amount.microgtu as i64,
            Some(memo.clone()),
        ),
        _ => Err(ApiError::UnsupportedOperationType(transaction_type_to_operation_type(Some(
            payload.transaction_type(),
        )))),
    }
}

fn operations_from_transfer_transaction(
    sender_addr: &AccountAddress,
    receiver_addr: &AccountAddress,
    amount_uccd: i64,
    memo: Option<Memo>,
) -> ApiResult<(Vec<Operation>, Option<Memo>)> {
    Ok((
        vec![
            Operation {
                operation_identifier: Box::new(OperationIdentifier::new(0)),
                related_operations:   None,
                _type:                OPERATION_TYPE_TRANSFER.to_string(),
                status:               None,
                account:              Some(Box::new(AccountIdentifier::new(
                    sender_addr.to_string(),
                ))),
                amount:               Some(Box::new(amount_from_uccd(-amount_uccd))),
                coin_change:          None,
                metadata:             None,
            },
            Operation {
                operation_identifier: Box::new(OperationIdentifier::new(1)),
                related_operations:   None,
                _type:                OPERATION_TYPE_TRANSFER.to_string(),
                status:               None,
                account:              Some(Box::new(AccountIdentifier::new(
                    receiver_addr.to_string(),
                ))),
                amount:               Some(Box::new(amount_from_uccd(amount_uccd))),
                coin_change:          None,
                metadata:             None,
            },
        ],
        memo,
    ))
}
