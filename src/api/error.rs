use concordium_rust_sdk::endpoints::RPCError;
use rosetta::models::*;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct UnsupportedNetworkIdentifier {
    provided: NetworkIdentifier,
    supported: Vec<NetworkIdentifier>,
}

impl UnsupportedNetworkIdentifier {
    pub fn new(provided: NetworkIdentifier, supported: Vec<NetworkIdentifier>) -> Self {
        UnsupportedNetworkIdentifier {
            provided,
            supported,
        }
    }
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("unsupported network identifier provided")]
    UnsupportedNetworkIdentifier(UnsupportedNetworkIdentifier),
    #[error("client RPC error")]
    ClientRpcError(RPCError),
}

impl warp::reject::Reject for ApiError {}

impl From<concordium_rust_sdk::endpoints::RPCError> for ApiError {
    fn from(err: RPCError) -> Self {
        ApiError::ClientRpcError(err)
    }
}
