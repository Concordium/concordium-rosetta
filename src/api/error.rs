use concordium_rust_sdk::endpoints::{QueryError, RPCError};
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

#[derive(Debug)]
pub enum InvalidBlockIdentifier {
    NoValues,
    InconsistentValues,
    InvalidHash,
    InvalidIndex,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("unsupported network identifier provided")]
    UnsupportedNetworkIdentifier(UnsupportedNetworkIdentifier),
    #[error("invalid block identifier")]
    InvalidBlockIdentifier(InvalidBlockIdentifier),
    #[error("invalid account address")]
    InvalidAccountAddress,
    #[error("invalid currency")]
    InvalidCurrency,
    #[error("no blocks matched")]
    NoBlocksMatched,
    #[error("multiple blocks matched")]
    MultipleBlocksMatched,
    #[error("no transactions matched")]
    NoTransactionsMatched,
    #[error("client RPC error")]
    ClientRpcError(RPCError),
    #[error("client query error")]
    ClientQueryError(QueryError),
    #[error("sub-accounts are not yet implemented")]
    SubAccountNotImplemented,
}

impl warp::reject::Reject for ApiError {}

impl From<RPCError> for ApiError {
    fn from(err: RPCError) -> Self {
        ApiError::ClientRpcError(err)
    }
}

impl From<QueryError> for ApiError {
    fn from(err: QueryError) -> Self {
        ApiError::ClientQueryError(err)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
