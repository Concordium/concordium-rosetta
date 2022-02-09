use concordium_rust_sdk::endpoints::RPCError;
use rosetta::models::*;
use serde::Serialize;
use serde_json::json;
use std::convert::Infallible;
use warp::reject::Reject;
use warp::{reject, reply, Rejection, Reply};

use crate::api::network::ApiError;
use crate::api::network::NetworkApi;

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
            ApiError::ClientRpcError(err) => match err {
                RPCError::CallError(err) => Ok(reply::json(&Error {
                    code: 10000,
                    message: "rpc: call error".to_string(),
                    description: None,
                    retriable: true,
                    details: Some(json!({ "error": err.to_string() })),
                })),
                RPCError::InvalidMetadata(err) => Ok(reply::json(&Error {
                    code: 10100,
                    message: "rpc: invalid metadata".to_string(),
                    description: None,
                    retriable: true,
                    details: Some(json!({ "error": err.to_string() })),
                })),
                RPCError::ParseError(err) => Ok(reply::json(&Error {
                    code: 10200,
                    message: "rpc: parse error".to_string(),
                    description: None,
                    retriable: true,
                    details: Some(json!({ "error": err.to_string() })),
                })),
            },
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

// TODO Can lift this function to remove the need for explicitly defining the above functions?
fn to_json(res: Result<impl Serialize, impl Reject>) -> Result<impl Reply, Rejection> {
    match res {
        Ok(val) => Ok(reply::json(&val)),
        Err(err) => Err(reject::custom(err)),
    }
}
