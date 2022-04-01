use crate::api::error::ApiError;
use rosetta::models::*;
use serde_json::{Map, Value};
use warp::{reply, Rejection, Reply};

pub async fn handle_rejection(rej: Rejection) -> Result<impl Reply, Rejection> {
    // Error code structure:
    //  1000 -  1999: invalid input
    //                 1000: unsupported field <name>
    //                       * subaccount
    //                       * max_fee
    //                       * suggested_fee_multiplier
    //                 1100: missing field <name>
    //                 1200: invalid value or identifier (type or format) <name, value>
    //                       * network identifier
    //                       * block identifier
    //                       * account identifier
    //                       * amount/currency
    //                       * signature
    //                       * encoded payload
    //                       * signed transaction
    //                       * construction options
    //                 1300: unsupported field value
    //                       * operation type
    //                 1400: inconsistent value
    //                       * operations (unexpected number, non-opposite amounts)
    //  2000 -  2999: identifier not resolved
    //                 2000: no matches <value>
    //                       * network identifier
    //                       * block identifier
    //                       * transaction hash
    //                       * account identifier
    //                 2100: multiple matches <value>
    //                       * block identifier
    //  9000 -  9999: internal error
    //                 9000: JSON encoding failed
    // 10000 - 19999: proxy error
    //                10000: client RPC error
    //                10100: client query error
    //                10200: transaction rejected
    match rej.find::<ApiError>() {
        None => Err(rej),
        Some(err) => {
            let e = match err {
                ApiError::UnsupportedFieldPresent(field_name) => {
                    invalid_input_unsupported_field_error(Some(field_name.to_string()))
                }
                ApiError::SubAccountNotImplemented => {
                    invalid_input_unsupported_field_error(Some("sub_account".to_string()))
                }
                ApiError::RequiredFieldMissing(name) => {
                    invalid_input_missing_field_error(Some(name.to_string()))
                }
                ApiError::InvalidAccountAddress(addr) => {
                    invalid_input_invalid_value_or_identifier_error(
                        Some("account address".to_string()),
                        None,
                        Some(addr.clone()),
                        Some("invalid format".to_string()),
                    )
                }
                ApiError::InvalidCurrency => invalid_input_invalid_value_or_identifier_error(
                    Some("currency".to_string()),
                    None,
                    None,
                    Some(
                        "only supported value is '{\"symbol\":\"CCD\",\"decimals\":6}'".to_string(),
                    ),
                ),
                ApiError::InvalidAmount(amount) => invalid_input_invalid_value_or_identifier_error(
                    Some("amount".to_string()),
                    None,
                    Some(amount.clone()),
                    None,
                ),
                ApiError::InvalidBlockIdentifier(err) => {
                    invalid_input_invalid_value_or_identifier_error(
                        Some("block identifier".to_string()),
                        None,
                        None,
                        Some(err.to_string()),
                    )
                }
                ApiError::InvalidSignature(sig, err) => {
                    invalid_input_invalid_value_or_identifier_error(
                        Some("signature".to_string()),
                        None,
                        Some(sig.clone()),
                        Some(err.to_string()),
                    )
                }
                ApiError::InvalidEncodedPayload => invalid_input_invalid_value_or_identifier_error(
                    Some("encoded transaction payload".to_string()),
                    None,
                    None,
                    None,
                ),
                ApiError::InvalidUnsignedTransaction => {
                    invalid_input_invalid_value_or_identifier_error(
                        Some("unsigned transaction".to_string()),
                        None,
                        None,
                        None,
                    )
                }
                ApiError::InvalidSignedTransaction => {
                    invalid_input_invalid_value_or_identifier_error(
                        Some("signed transaction".to_string()),
                        None,
                        None,
                        None,
                    )
                }
                ApiError::InvalidConstructionOptions => {
                    invalid_input_invalid_value_or_identifier_error(
                        Some("construction options".to_string()),
                        None,
                        None,
                        None,
                    )
                }
                ApiError::InvalidPayloadsMetadata => {
                    invalid_input_invalid_value_or_identifier_error(
                        Some("payloads metadata".to_string()),
                        None,
                        None,
                        None,
                    )
                }
                ApiError::UnsupportedOperationType(name) => invalid_input_unsupported_value_error(
                    Some("operation type".to_string()),
                    Some(name.clone()),
                ),
                ApiError::InconsistentOperations(err) => invalid_input_inconsistent_value_error(
                    Some("operations".to_string()),
                    Some(err.clone()),
                ),
                ApiError::UnsupportedNetworkIdentifier => {
                    identifier_not_resolved_no_matches_error(Some("network_identifier".to_string()))
                }
                ApiError::NoBlocksMatched => {
                    identifier_not_resolved_no_matches_error(Some("block_identifier".to_string()))
                }
                ApiError::NoTransactionsMatched => identifier_not_resolved_no_matches_error(Some(
                    "transaction_identifier".to_string(),
                )),
                ApiError::MultipleBlocksMatched => identifier_not_resolved_multiple_matches_error(
                    Some("block_identifier".to_string()),
                ),
                ApiError::JsonEncodingFailed(field_name, err) => {
                    internal_json_encoding_failed_error(
                        Some(field_name.clone()),
                        Some(err.to_string()),
                    )
                }
                ApiError::ClientRpcError(err) => proxy_client_rpc_error(Some(err.to_string())),
                ApiError::ClientQueryError(err) => proxy_client_query_error(Some(err.to_string())),
                ApiError::TransactionNotAccepted => proxy_transaction_rejected(),
            };
            Ok(reply::json(&e))
        }
    }
}

fn key_value_pairs(pairs: &Vec<Option<(String, String)>>) -> Option<Value> {
    let mut m = Map::new();
    for pair in pairs {
        if let Some((k, v)) = pair {
            m.insert(k.clone(), Value::String(v.clone()));
        }
    }
    if m.is_empty() {
        None
    } else {
        Some(Value::Object(m))
    }
}

fn key_value_pair(key: &str, value: Option<String>) -> Option<(String, String)> {
    value.map(|v| (key.to_string(), v))
}

pub fn invalid_input_unsupported_field_error(field_name: Option<String>) -> Error {
    Error {
        code: 1000,
        message: "invalid input: field is not supported".to_string(),
        description: Some("The provided field is not supported.".to_string()),
        retriable: false,
        details: key_value_pairs(&vec![key_value_pair("field", field_name)]),
    }
}

pub fn invalid_input_missing_field_error(field_name: Option<String>) -> Error {
    Error {
        code: 1100,
        message: "invalid input: required field is missing".to_string(),
        description: Some("The required field is not provided.".to_string()),
        retriable: false,
        details: key_value_pairs(&vec![key_value_pair("field", field_name)]),
    }
}

pub fn invalid_input_invalid_value_or_identifier_error(
    name: Option<String>,
    type_: Option<String>,
    value: Option<String>,
    msg: Option<String>,
) -> Error {
    Error {
        code: 1200,
        message: "invalid input: invalid value or identifier".to_string(),
        description: Some(
            "The provided value or identifier is incorrectly typed or formatted.".to_string(),
        ),
        retriable: false,
        details: key_value_pairs(&vec![
            key_value_pair("name", name),
            key_value_pair("value", value),
            key_value_pair("type", type_),
            key_value_pair("message", msg),
        ]),
    }
}

pub fn invalid_input_unsupported_value_error(name: Option<String>, value: Option<String>) -> Error {
    Error {
        code: 1300,
        message: "invalid input: unsupported value".to_string(),
        description: Some("".to_string()),
        retriable: false,
        details: key_value_pairs(&vec![
            key_value_pair("name", name),
            key_value_pair("value", value),
        ]),
    }
}

pub fn invalid_input_inconsistent_value_error(
    field_name: Option<String>,
    msg: Option<String>,
) -> Error {
    Error {
        code: 1400,
        message: "invalid input: inconsistent value".to_string(),
        description: Some(
            "The provided value does not satisfy all consistency requirements.".to_string(),
        ),
        retriable: false,
        details: key_value_pairs(&vec![
            key_value_pair("field", field_name),
            key_value_pair("message", msg),
        ]),
    }
}

pub fn identifier_not_resolved_no_matches_error(identifier_type: Option<String>) -> Error {
    Error {
        code: 2000,
        message: "identifier not resolved: no matches".to_string(),
        description: Some("The provided identifier did not match any objects.".to_string()),
        retriable: false,
        details: key_value_pairs(&vec![key_value_pair("type", identifier_type)]),
    }
}

pub fn identifier_not_resolved_multiple_matches_error(identifier_type: Option<String>) -> Error {
    Error {
        code: 2100,
        message: "identifier not resolved: multiple matches".to_string(),
        description: Some("The provided identifier matched multiple objects.".to_string()),
        retriable: false,
        details: key_value_pairs(&vec![key_value_pair("type", identifier_type)]),
    }
}

pub fn internal_json_encoding_failed_error(
    field_name: Option<String>,
    err: Option<String>,
) -> Error {
    Error {
        code: 9000,
        message: "internal error: JSON encoding failed".to_string(),
        description: Some("JSON encoding failed.".to_string()),
        retriable: false,
        details: key_value_pairs(&vec![
            key_value_pair("field", field_name),
            key_value_pair("message", err),
        ]),
    }
}

pub fn proxy_client_rpc_error(err: Option<String>) -> Error {
    Error {
        code: 10000,
        message: "proxy error: node RPC error".to_string(),
        description: Some("Some interaction with the node failed with an 'RPC error'.".to_string()),
        retriable: true,
        details: key_value_pairs(&vec![key_value_pair("message", err)]),
    }
}

pub fn proxy_client_query_error(err: Option<String>) -> Error {
    Error {
        code: 10100,
        message: "proxy error: node query error".to_string(),
        description: Some(
            "Some interaction with the node failed with a 'query error'.".to_string(),
        ),
        retriable: true,
        details: key_value_pairs(&vec![key_value_pair("message", err)]),
    }
}

pub fn proxy_transaction_rejected() -> Error {
    Error {
        code: 10200,
        message: "proxy error: node rejected transaction".to_string(),
        description: Some("The submitted transaction was rejected by the node.".to_string()),
        retriable: false,
        details: None,
    }
}