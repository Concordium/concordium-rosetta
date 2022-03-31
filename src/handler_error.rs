use concordium_rust_sdk::endpoints::{QueryError, RPCError};
use rosetta::models::*;
use serde_json::{Map, Value};
use warp::{reply, Rejection, Reply};
use crate::api::error::ApiError;

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
                    invalid_input_unsupported_field_error(field_name.as_str())
                }
                ApiError::SubAccountNotImplemented => {
                    invalid_input_unsupported_field_error("sub_account")
                }
                ApiError::RequiredFieldMissing(name) => {
                    invalid_input_missing_field_error(name.as_str())
                }
                ApiError::InvalidAccountAddress(addr) => {
                    invalid_input_invalid_value_or_identifier_error("account address", None, Some(addr.clone()), Some("invalid format".to_string()))
                }
                ApiError::InvalidCurrency => {
                    invalid_input_invalid_value_or_identifier_error("currency", None, None, Some("only supported value is '{\"symbol\":\"CCD\",\"decimals\":6}'".to_string()))
                }
                ApiError::InvalidAmount(amount) => {
                    invalid_input_invalid_value_or_identifier_error("amount", None, Some(amount.clone()), None)
                }
                ApiError::InvalidBlockIdentifier(_) => {
                    invalid_input_invalid_value_or_identifier_error("block identifier", None, None, None)
                }
                ApiError::InvalidSignature(sig, err) => {
                    invalid_input_invalid_value_or_identifier_error("signature", None, Some(sig.clone()), Some(err.to_string()))
                }
                ApiError::InvalidEncodedPayload => {
                    invalid_input_invalid_value_or_identifier_error("encoded transaction payload", None, None, None)
                }
                ApiError::InvalidUnsignedTransaction => {
                    invalid_input_invalid_value_or_identifier_error("unsigned transaction", None, None, None)
                }
                ApiError::InvalidSignedTransaction => {
                    invalid_input_invalid_value_or_identifier_error("signed transaction", None, None, None)
                }
                ApiError::InvalidConstructionOptions => {
                    invalid_input_invalid_value_or_identifier_error("construction options", None, None, None)
                }
                ApiError::InvalidPayloadsMetadata => {
                    invalid_input_invalid_value_or_identifier_error("payloads metadata", None, None, None)
                }
                ApiError::UnsupportedOperationType(name) => {
                    invalid_input_unsupported_value_error("operation type", name.as_str())
                }
                ApiError::InconsistentOperations(_) => {
                    invalid_input_inconsistent_value_error("operations")
                }
                ApiError::UnsupportedNetworkIdentifier(_) => {
                    identifier_not_resolved_no_matches_error("network_identifier")
                }
                ApiError::NoBlocksMatched => {
                    identifier_not_resolved_no_matches_error("block_identifier")
                }
                ApiError::NoTransactionsMatched => {
                    identifier_not_resolved_no_matches_error("transaction_identifier")
                }
                ApiError::MultipleBlocksMatched => {
                    identifier_not_resolved_multiple_matches_error("block_identifier")
                }
                ApiError::JsonEncodingFailed(field_name, err) => {
                    internal_json_encoding_failed_error(field_name, err)
                }
                ApiError::ClientRpcError(err) => {
                    proxy_client_rpc_error(err)
                }
                ApiError::ClientQueryError(err) => {
                    proxy_client_query_error(err)
                }
                ApiError::TransactionNotAccepted => {
                    proxy_transaction_rejected()
                }
            };
            Ok(reply::json(&e))
        }
    }
}

fn key_value(k: &str, v: &str) -> Value {
    let mut details = Map::new();
    details.insert(k.to_string(), Value::String(v.to_string()));
    Value::Object(details)
}

fn key_value2(k1: &str, v1: &str, k2: &str, v2: &str) -> Value {
    let mut details = Map::new();
    details.insert(k1.to_string(), Value::String(v1.to_string()));
    details.insert(k2.to_string(), Value::String(v2.to_string()));
    Value::Object(details)
}

fn invalid_input_unsupported_field_error(field_name: &str) -> Error {
    Error {
        code: 1000,
        message: "invalid input: field is not supported".to_string(),
        description: Some("The provided field is not supported.".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn invalid_input_missing_field_error(field_name: &str) -> Error {
    Error {
        code: 1100,
        message: "invalid input: required field is missing".to_string(),
        description: Some("The required field is not provided.".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn invalid_input_invalid_value_or_identifier_error(name: &str, type_: Option<String>, value: Option<String>, msg: Option<String>) -> Error {
    let mut details = Map::new();
    details.insert("name".to_string(), Value::String(name.to_string()));
    if let Some(t) = type_ {
        details.insert("type".to_string(), Value::String(t.to_string()));
    }
    if let Some(v) = value {
        details.insert("value".to_string(), Value::String(v.to_string()));
    }
    if let Some(m) = msg {
        details.insert("message".to_string(), Value::String(m.to_string()));
    }
    Error {
        code: 1200,
        message: "invalid input: invalid value or identifier".to_string(),
        description: Some("The provided value or identifier is incorrectly typed or formatted.".to_string()),
        retriable: false,
        details: Some(Value::Object(details)),
    }
}

fn invalid_input_unsupported_value_error(name: &str, value: &str) -> Error {
    Error {
        code: 1300,
        message: "invalid input: unsupported value".to_string(),
        description: Some("".to_string()),
        retriable: false,
        details: Some(key_value2("name", name, "value", value)),
    }
}

fn invalid_input_inconsistent_value_error(field_name: &str) -> Error {
    Error {
        code: 1400,
        message: "invalid input: inconsistent value".to_string(),
        description: Some("The provided value does not satisfy all consistency requirements.".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn identifier_not_resolved_no_matches_error(field_name: &str) -> Error {
    Error {
        code: 2000,
        message: "".to_string(),
        description: Some("".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn identifier_not_resolved_multiple_matches_error(field_name: &str) -> Error {
    Error {
        code: 2100,
        message: "".to_string(),
        description: Some("".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn internal_json_encoding_failed_error(field_name: &str, err: &serde_json::Error) -> Error {
    Error {
        code: 9000,
        message: "".to_string(),
        description: Some("".to_string()),
        retriable: false,
        details: Some(key_value2("field", field_name, "message", err.to_string().as_str())),
    }
}

fn proxy_client_rpc_error(err: &RPCError) -> Error {
    Error {
        code: 10000,
        message: "proxy error: node RPC error".to_string(),
        description: Some("Some interaction with the node failed with an 'RPC error'.".to_string()),
        retriable: true,
        details: Some(key_value("message", err.to_string().as_str())),
    }
}

fn proxy_client_query_error(err: &QueryError) -> Error {
    Error {
        code: 10100,
        message: "proxy error: node query error".to_string(),
        description: Some("Some interaction with the node failed with a 'query error'.".to_string()),
        retriable: true,
        details: Some(key_value("message", err.to_string().as_str())),
    }
}

fn proxy_transaction_rejected() -> Error {
    Error {
        code: 10200,
        message: "proxy error: node rejected transaction".to_string(),
        description: Some("The submitted transaction was rejected by the node.".to_string()),
        retriable: false,
        details: None,
    }
}
