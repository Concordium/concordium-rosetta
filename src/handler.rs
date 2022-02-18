use crate::api::block::BlockApi;
use crate::AccountApi;
use concordium_rust_sdk::endpoints::RPCError;
use rosetta::models::*;
use serde::Serialize;
use serde_json::json;
use std::convert::Infallible;
use warp::reject::Reject;
use warp::{reject, reply, Rejection, Reply};

use crate::api::error::{ApiError, InvalidBlockIdentifier};
use crate::api::network::NetworkApi;
use crate::handler::NotImplemented::*;

enum NotImplemented {
    EndpointNotImplemented(String),
    ParameterNotImplemented(String),
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

pub async fn handle_rejection(rej: Rejection) -> Result<impl Reply, Rejection> {
    // Error code ranges:
    //         <  1000 : HTTP status codes
    //    1000 -  9999 : Application errors
    //   10000 - 19990 : Client errors
    if let Some(err) = rej.find::<ApiError>() {
        match err {
            ApiError::UnsupportedNetworkIdentifier(err) => Ok(reply::json(&Error {
                code: 1000,
                message: "unsupported network identifier".to_string(),
                description: Some("The provided network identifier does not identify a network that is supported by this server.".to_string()),
                retriable: true,
                details: Some(serde_json::to_value(err).unwrap()),
            })),
            ApiError::InvalidBlockIdentifier(reason) =>
                match reason {
                    InvalidBlockIdentifier::NoValues =>
                        Ok(reply::json(&Error {
                            code: 1010,
                            message: "missing block identifier".to_string(),
                            description: Some("TODO".to_string()),
                            retriable: false,
                            details: None,
                        })),
                    InvalidBlockIdentifier::InconsistentValues =>
                        Ok(reply::json(&Error {
                            code: 1011,
                            message: "inconsistent block identifier".to_string(),
                            description: Some("TODO".to_string()),
                            retriable: false,
                            details: None,
                        })),
                    InvalidBlockIdentifier::InvalidHash =>
                        Ok(reply::json(&Error {
                            code: 1012,
                            message: "invalid block hash".to_string(),
                            description: Some("TODO".to_string()),
                            retriable: false,
                            details: None,
                        })),
                    InvalidBlockIdentifier::InvalidIndex =>
                        Ok(reply::json(&Error {
                            code: 1013,
                            message: "invalid block index".to_string(),
                            description: Some("TODO".to_string()),
                            retriable: false,
                            details: None,
                        })),
                }
            ApiError::InvalidAccountAddress => Ok(reply::json(&Error {
                code: 1020,
                message: "invalid account address".to_string(),
                description: Some("TODO".to_string()),
                retriable: false,
                details: None, // TODO
            })),
            ApiError::InvalidCurrency => Ok(reply::json(&Error {
                code: 1030,
                message: "invalid currency".to_string(),
                description: Some("TODO".to_string()),
                retriable: false,
                details: None, // TODO
            })),
            ApiError::NoBlocksMatched => Ok(reply::json(&Error {
                code: 1040,
                message: "no blocks matched".to_string(),
                description: Some("TODO".to_string()),
                retriable: false,
                details: None, // TODO
            })),
            ApiError::MultipleBlocksMatched => Ok(reply::json(&Error {
                code: 1050,
                message: "multiple blocks matched".to_string(),
                description: Some("TODO".to_string()),
                retriable: false,
                details: None, // TODO
            })),
            ApiError::NoTransactionsMatched => Ok(reply::json(&Error {
                code: 1060,
                message: "no transactions matched".to_string(),
                description: Some("TODO".to_string()),
                retriable: false,
                details: None, // TODO
            })),
            ApiError::ClientRpcError(err) => match err {
                RPCError::CallError(err) => Ok(reply::json(&Error {
                    code: 10000,
                    message: "sdk: rpc: call error".to_string(),
                    description: None,
                    retriable: true,
                    details: Some(json!({ "error": err.to_string() })),
                })),
                RPCError::InvalidMetadata(err) => Ok(reply::json(&Error {
                    code: 10100,
                    message: "sdk: rpc: invalid metadata".to_string(),
                    description: None,
                    retriable: false,
                    details: Some(json!({ "error": err.to_string() })),
                })),
                RPCError::ParseError(err) => Ok(reply::json(&Error {
                    code: 10200,
                    message: "sdk: rpc: parse error".to_string(),
                    description: None,
                    retriable: true,
                    details: Some(json!({ "error": err.to_string() })),
                })),
            },
            ApiError::ClientQueryError(err) => Ok(reply::json(&Error {
                code: 11000,
                message: "sdk: query error".to_string(),
                description: None,
                retriable: true,
                details: Some(json!({ "error": err.to_string() })),
            })),
            ApiError::SubAccountNotImplemented => not_implemented(ParameterNotImplemented("sub_account".to_string()))
        }
    } else {
        Err(rej)
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

pub async fn block_transaction(api: BlockApi, req: BlockTransactionRequest) -> Result<impl Reply, Rejection> {
    to_json(api.block_transaction(req).await)
}

// TODO Can lift this function to remove the need for explicitly defining the above functions?
fn to_json(res: Result<impl Serialize, impl Reject>) -> Result<impl Reply, Rejection> {
    match res {
        Ok(val) => Ok(reply::json(&val)),
        Err(err) => Err(reject::custom(err)),
    }
}
