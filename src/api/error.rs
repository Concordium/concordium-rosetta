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

#[derive(Debug)]
pub enum InvalidSignature {
    MissingSeparator,
    MissingIndexSeparator,
    InvalidCredentialIndex(String),
    InvalidKeyIndex(String),
    InvalidSignatureHexBytes(String),
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
    #[error("invalid amount")]
    InvalidAmount,
    #[error("no blocks matched")]
    NoBlocksMatched,
    #[error("multiple blocks matched")]
    MultipleBlocksMatched,
    #[error("no transactions matched")]
    NoTransactionsMatched,
    #[error("unsupported combination of operations")]
    UnsupportedCombinationOfOperations,
    #[error("unsupported operation type")]
    UnsupportedOperationType(String),
    #[error("field is not supported")]
    UnsupportedFieldPresent(String),
    #[error("required field is missing")]
    RequiredFieldMissing(String),
    #[error("client RPC error")]
    ClientRpcError(RPCError),
    #[error("client query error")]
    ClientQueryError(QueryError),
    #[error("json encode or decode error")]
    SerdeError(serde_json::Error),
    #[error("sub-accounts are not yet implemented")]
    SubAccountNotImplemented,
    #[error("invalid operations")]
    InconsistentOperations(String),
    #[error("invalid encoded payload")]
    InvalidEncodedPayload,
    #[error("invalid signature")]
    InvalidSignature(String, InvalidSignature),
    #[error("transaction not accepted by node")]
    TransactionNotAccepted,
}

impl warp::reject::Reject for ApiError {}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::SerdeError(err)
    }
}

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
