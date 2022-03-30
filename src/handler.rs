use crate::api::block::BlockApi;
use crate::api::construction::ConstructionApi;
use crate::AccountApi;
use concordium_rust_sdk::endpoints::{QueryError, RPCError};
use rosetta::models::*;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::convert::Infallible;
use warp::reject::Reject;
use warp::{reject, reply, Rejection, Reply};

use crate::api::error::{ApiError};
use crate::api::network::NetworkApi;
use crate::handler::NotImplemented::*;

enum NotImplemented {
    EndpointNotImplemented(String),
    ParameterNotImplemented(String),
}

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
                    err_invalid_input_unsupported_field(field_name.as_str())
                }
                ApiError::SubAccountNotImplemented => {
                    err_invalid_input_unsupported_field("sub_account")
                }
                ApiError::RequiredFieldMissing(name) => {
                    err_invalid_input_missing_field(name.as_str())
                }
                ApiError::InvalidAccountAddress(addr) => {
                    err_invalid_input_invalid_value_or_identifier("account address", None, Some(addr.clone()), Some("invalid format".to_string()))
                }
                ApiError::InvalidCurrency => {
                    err_invalid_input_invalid_value_or_identifier("currency", None, None, Some("only supported value is '{\"symbol\":\"CCD\",\"decimals\":6}'".to_string()))
                }
                ApiError::InvalidAmount(amount) => {
                    err_invalid_input_invalid_value_or_identifier("amount", None, Some(amount.clone()), None)
                }
                ApiError::InvalidBlockIdentifier(_) => {
                    err_invalid_input_invalid_value_or_identifier("block identifier", None, None, None)
                }
                ApiError::InvalidSignature(sig, err) => {
                    err_invalid_input_invalid_value_or_identifier("signature", None, Some(sig.clone()), Some(err.to_string()))
                }
                ApiError::InvalidEncodedPayload => {
                    err_invalid_input_invalid_value_or_identifier("encoded transaction payload", None, None, None)
                }
                ApiError::InvalidUnsignedTransaction => {
                    err_invalid_input_invalid_value_or_identifier("unsigned transaction", None, None, None)
                }
                ApiError::InvalidSignedTransaction => {
                    err_invalid_input_invalid_value_or_identifier("signed transaction", None, None, None)
                }
                ApiError::InvalidConstructionOptions => {
                    err_invalid_input_invalid_value_or_identifier("construction options", None, None, None)
                }
                ApiError::InvalidPayloadsMetadata => {
                    err_invalid_input_invalid_value_or_identifier("payloads metadata", None, None, None)
                }
                ApiError::UnsupportedOperationType(name) => {
                    err_invalid_input_unsupported_value("operation type", name.as_str())
                }
                // ApiError::UnsupportedCombinationOfOperations => {
                //     err_invalid_input_inconsistent_value("operations")
                // }
                ApiError::InconsistentOperations(_) => {
                    err_invalid_input_inconsistent_value("operations")
                }
                ApiError::UnsupportedNetworkIdentifier(_) => {
                    err_identifier_not_resolved_no_matches("network_identifier")
                }
                ApiError::NoBlocksMatched => {
                    err_identifier_not_resolved_no_matches("block_identifier")
                }
                ApiError::NoTransactionsMatched => {
                    err_identifier_not_resolved_no_matches("transaction_identifier")
                }
                ApiError::MultipleBlocksMatched => {
                    err_identifier_not_resolved_multiple_matches("block_identifier")
                }
                ApiError::JsonEncodingFailed(field_name, err) => {
                    err_internal_json_encoding_failed( field_name, err)
                }
                ApiError::ClientRpcError(err) => {
                    err_proxy_client_rpc_error(err)
                }
                ApiError::ClientQueryError(err) => {
                    err_proxy_client_query_error(err)
                }
                ApiError::TransactionNotAccepted => {
                    err_proxy_transaction_rejected()
                }
                // _ =>
                //     Error {
                //         code: 9999,
                //         message: "unknown".to_string(),
                //         description: None,
                //         retriable: true,
                //         details: None,
                //     },
            };
            Ok(reply::json(&e))
            
            
            // match err {
            //     ApiError::UnsupportedNetworkIdentifier(err) => Ok(reply::json(&Error {
            //         code: 1000,
            //         message: "unsupported network identifier".to_string(),
            //         description: Some("The provided network identifier does not identify a network that is supported by this server.".to_string()),
            //         retriable: true,
            //         details: Some(serde_json::to_value(err).unwrap()),
            //     })),
            //     ApiError::InvalidBlockIdentifier(reason) =>
            //         match reason {
            //             InvalidBlockIdentifier::NoValues =>
            //                 Ok(reply::json(&Error {
            //                     code: 1010,
            //                     message: "missing block identifier".to_string(),
            //                     description: Some("TODO".to_string()),
            //                     retriable: false,
            //                     details: None,
            //                 })),
            //             InvalidBlockIdentifier::InconsistentValues =>
            //                 Ok(reply::json(&Error {
            //                     code: 1011,
            //                     message: "inconsistent block identifier".to_string(),
            //                     description: Some("TODO".to_string()),
            //                     retriable: false,
            //                     details: None,
            //                 })),
            //             InvalidBlockIdentifier::InvalidHash =>
            //                 Ok(reply::json(&Error {
            //                     code: 1012,
            //                     message: "invalid block hash".to_string(),
            //                     description: Some("TODO".to_string()),
            //                     retriable: false,
            //                     details: None,
            //                 })),
            //             InvalidBlockIdentifier::InvalidIndex =>
            //                 Ok(reply::json(&Error {
            //                     code: 1013,
            //                     message: "invalid block index".to_string(),
            //                     description: Some("TODO".to_string()),
            //                     retriable: false,
            //                     details: None,
            //                 })),
            //         }
            //     ApiError::InvalidAccountAddress(_) => Ok(reply::json(&Error {
            //         code: 1020,
            //         message: "invalid account address".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::InvalidCurrency => Ok(reply::json(&Error {
            //         code: 1030,
            //         message: "invalid currency".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::NoBlocksMatched => Ok(reply::json(&Error {
            //         code: 1040,
            //         message: "no blocks matched".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::MultipleBlocksMatched => Ok(reply::json(&Error {
            //         code: 1050,
            //         message: "multiple blocks matched".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::NoTransactionsMatched => Ok(reply::json(&Error {
            //         code: 1060,
            //         message: "no transactions matched".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::UnsupportedCombinationOfOperations => Ok(reply::json(&Error {
            //         code: 1080,
            //         message: "unsupported combination of operations".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::UnsupportedOperationType(_) => Ok(reply::json(&Error {
            //         code: 1090,
            //         message: "unsupported operation type".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::UnsupportedFieldPresent(_) => Ok(reply::json(&Error {
            //         code: 1091,
            //         message: "field is not supported".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::InconsistentOperations(_) => Ok(reply::json(&Error {
            //         code: 1110,
            //         message: "invalid operations".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::InvalidAmount(_) => Ok(reply::json(&Error {
            //         code: 1115,
            //         message: "invalid amount".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::InvalidEncodedPayload => Ok(reply::json(&Error {
            //         code: 1116,
            //         message: "invalid encoded payload".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::InvalidSignature(sig, err) => Ok(reply::json(&Error {
            //         code: 1117,
            //         message: "invalid signature".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::RequiredFieldMissing(_) => Ok(reply::json(&Error {
            //         code: 1120,
            //         message: "required field is missing".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::TransactionNotAccepted => Ok(reply::json(&Error {
            //         code: 1130,
            //         message: "transaction not accepted by node".to_string(),
            //         description: Some("TODO".to_string()),
            //         retriable: false,
            //         details: None, // TODO
            //     })),
            //     ApiError::ClientRpcError(err) => match err {
            //         RPCError::CallError(err) => Ok(reply::json(&Error {
            //             code: 10000,
            //             message: "sdk: rpc: call error".to_string(),
            //             description: None,
            //             retriable: true,
            //             details: Some(json!({ "error": err.to_string() })),
            //         })),
            //         RPCError::InvalidMetadata(err) => Ok(reply::json(&Error {
            //             code: 10100,
            //             message: "sdk: rpc: invalid metadata".to_string(),
            //             description: None,
            //             retriable: false,
            //             details: Some(json!({ "error": err.to_string() })),
            //         })),
            //         RPCError::ParseError(err) => Ok(reply::json(&Error {
            //             code: 10200,
            //             message: "sdk: rpc: parse error".to_string(),
            //             description: None,
            //             retriable: true,
            //             details: Some(json!({ "error": err.to_string() })),
            //         })),
            //     },
            //     ApiError::ClientQueryError(err) => Ok(reply::json(&Error {
            //         code: 11000,
            //         message: "sdk: query error".to_string(),
            //         description: None,
            //         retriable: true,
            //         details: Some(json!({ "error": err.to_string() })),
            //     })),
            //     // ApiError::SerdeError(err) => Ok(reply::json(&Error {
            //     //     code: 12000,
            //     //     message: "JSON encode or decode error".to_string(),
            //     //     description: None,
            //     //     retriable: true,
            //     //     details: Some(json!({ "error": err.to_string() })),
            //     // })),
            //     ApiError::SubAccountNotImplemented => not_implemented(ParameterNotImplemented("sub_account".to_string())),
            // }
        }
    }
}

fn not_implemented(err: NotImplemented) -> Result<reply::Json, Rejection> {
    let details = match err {
        NotImplemented::EndpointNotImplemented(e) => json!({ "endpoint": e }),
        NotImplemented::ParameterNotImplemented(p) => json!({ "parameter": p }),
    };
    Ok(reply::json(&Error {
        code: 9000,
        message: "feature is not implemented".to_string(),
        description: None,
        retriable: false,
        details: Some(details),
    }))
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

fn err_invalid_input_unsupported_field(field_name: &str) -> Error {
    Error {
        code: 1000,
        message: "invalid input: field is not supported".to_string(),
        description: Some("The provided field is not supported.".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn err_invalid_input_missing_field(field_name: &str) -> Error {
    Error {
        code: 1100,
        message: "invalid input: required field is missing".to_string(),
        description: Some("The required field is not provided.".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn err_invalid_input_invalid_value_or_identifier(name: &str, type_: Option<String>, value: Option<String>, msg: Option<String>) -> Error {
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

fn err_invalid_input_unsupported_value(name: &str, value: &str) -> Error {
    Error {
        code: 1300,
        message: "invalid input: unsupported value".to_string(),
        description: Some("".to_string()),
        retriable: false,
        details: Some(key_value2("name", name, "value", value)),
    }
}

fn err_invalid_input_inconsistent_value(field_name: &str) -> Error {
    Error {
        code: 1400,
        message: "invalid input: inconsistent value".to_string(),
        description: Some("The provided value does not satisfy all consistency requirements.".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn err_identifier_not_resolved_no_matches(field_name: &str) -> Error {
    Error {
        code: 2000,
        message: "".to_string(),
        description: Some("".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn err_identifier_not_resolved_multiple_matches(field_name: &str) -> Error {
    Error {
        code: 2100,
        message: "".to_string(),
        description: Some("".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn err_internal_json_encoding_failed(field_name: &str, err: &serde_json::Error) -> Error {
    Error {
        code: 9000,
        message: "".to_string(),
        description: Some("".to_string()),
        retriable: false,
        details: Some(key_value("field", field_name)),
    }
}

fn err_proxy_client_rpc_error(field_name: &RPCError) -> Error {
    Error {
        code: 10000,
        message: "proxy error: node RPC error".to_string(),
        description: Some("Some interaction with the node failed with an 'RPC error'.".to_string()),
        retriable: true,
        details: None,
    }
}

fn err_proxy_client_query_error(field_name: &QueryError) -> Error {
    Error {
        code: 10100,
        message: "proxy error: node query error".to_string(),
        description: Some("Some interaction with the node failed with a 'query error'.".to_string()),
        retriable: true,
        details: None,
    }
}

fn err_proxy_transaction_rejected() -> Error {
    Error {
        code: 10200,
        message: "proxy error: transaction rejected".to_string(),
        description: Some("The submitted transaction was rejected by the node.".to_string()),
        retriable: false,
        details: None,
    }
}

pub async fn network_list(api: NetworkApi, _: MetadataRequest) -> Result<impl Reply, Infallible> {
    Ok(reply::json(&api.network_list()))
}

pub async fn network_options(
    api: NetworkApi,
    req: NetworkRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.network_options(req).await)
}

pub async fn network_status(api: NetworkApi, req: NetworkRequest) -> Result<impl Reply, Rejection> {
    to_json(api.network_status(req).await)
}

pub async fn account_balance(
    api: AccountApi,
    req: AccountBalanceRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.account_balance(req).await)
}

pub async fn account_coins(_: AccountCoinsRequest) -> Result<impl Reply, Rejection> {
    not_implemented(EndpointNotImplemented("/account/coins".to_string()))
}

pub async fn block(api: BlockApi, req: BlockRequest) -> Result<impl Reply, Rejection> {
    to_json(api.block(req).await)
}

pub async fn block_transaction(
    api: BlockApi,
    req: BlockTransactionRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.block_transaction(req).await)
}

pub async fn construction_derive(_: AccountCoinsRequest) -> Result<impl Reply, Rejection> {
    not_implemented(EndpointNotImplemented("/construction/derive".to_string()))
}

pub async fn construction_preprocess(
    api: ConstructionApi,
    req: ConstructionPreprocessRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.preprocess(req).await)
}

pub async fn construction_metadata(
    api: ConstructionApi,
    req: ConstructionMetadataRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.metadata(req).await)
}

pub async fn construction_payloads(
    api: ConstructionApi,
    req: ConstructionPayloadsRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.payloads(req).await)
}

pub async fn construction_parse(
    api: ConstructionApi,
    req: ConstructionParseRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.parse(req).await)
}

pub async fn construction_combine(
    api: ConstructionApi,
    req: ConstructionCombineRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.combine(req).await)
}

pub async fn construction_submit(
    api: ConstructionApi,
    req: ConstructionSubmitRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.submit(req).await)
}

pub async fn construction_hash(
    api: ConstructionApi,
    req: ConstructionHashRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.hash(req).await)
}

// TODO Can lift this function to remove the need for explicitly defining the above functions?
fn to_json(res: Result<impl Serialize, impl Reject>) -> Result<impl Reply, Rejection> {
    match res {
        Ok(val) => Ok(reply::json(&val)),
        Err(err) => Err(reject::custom(err)),
    }
}
