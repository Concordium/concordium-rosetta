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
pub enum InvalidSignatureError {
    #[error("separator '{0}' is missing")]
    MissingSeparator(String),
    #[error("index separator '{0}' is missing")]
    MissingIndexSeparator(String),
    #[error("invalid credential index")]
    InvalidCredentialIndex(String),
    #[error("invalid key index '{0}'")]
    InvalidKeyIndex(String),
    #[error("invalid signature hex bytes '{0}'")]
    InvalidSignatureHexBytes(String),
}

#[derive(Error, Debug)]
pub enum ApiError {
    // Invalid input: Unsupported field.
    #[error("field is not supported")]
    UnsupportedFieldPresent(String),
    #[error("sub-accounts are not supported")]
    SubAccountNotImplemented,
    
    // Invalid input: Missing field.
    #[error("required field is missing")]
    RequiredFieldMissing(String),

    // Invalid input: Invalid value or identifier (type or format).
    #[error("invalid account address")]
    InvalidAccountAddress(String),
    #[error("invalid currency")]
    InvalidCurrency,
    #[error("invalid amount")]
    InvalidAmount(String),
    #[error("invalid block identifier")]
    InvalidBlockIdentifier(InvalidBlockIdentifier),
    #[error("invalid signature")]
    InvalidSignature(String, InvalidSignatureError),
    #[error("invalid encoded transaction payload")]
    InvalidEncodedPayload,
    #[error("invalid unsigned transaction")]
    InvalidUnsignedTransaction,
    #[error("invalid signed transaction")]
    InvalidSignedTransaction,
    #[error("invalid construction options")]
    InvalidConstructionOptions,
    #[error("invalid payloads metadata")]
    InvalidPayloadsMetadata,
    // #[error("json encode or decode error")]
    // SerdeError(serde_json::Error),
    
    // Invalid input: Unsupported field value.
    #[error("unsupported operation type")]
    UnsupportedOperationType(String),
    
    // Invalid input: Inconsistent value.
    // #[error("unsupported combination of operations")]
    // UnsupportedCombinationOfOperations,
    #[error("inconsistent operations")]
    InconsistentOperations(String),
    
    // Identifier not resolved: Unresolved identifier.
    #[error("unsupported network identifier provided")]
    UnsupportedNetworkIdentifier(UnsupportedNetworkIdentifier),
    #[error("no blocks matched")]
    NoBlocksMatched,
    #[error("no transactions matched")]
    NoTransactionsMatched,
    
    // Identifier not resolved: Ambiguous identifier.
    #[error("multiple blocks matched")]
    MultipleBlocksMatched,

    // Internal errors.
    #[error("transaction not accepted by node")]
    TransactionNotAccepted,
    #[error("JSON encoding failed")]
    JsonEncodingFailed(String, serde_json::Error),

    // Proxy errors.
    #[error("client RPC error")]
    ClientRpcError(RPCError),
    #[error("client query error")]
    ClientQueryError(QueryError),
}

impl warp::reject::Reject for ApiError {}

impl From<RPCError> for ApiError {
    fn from(err: RPCError) -> Self {
        ApiError::ClientRpcError(err)
    }
}

impl From<QueryError> for ApiError {
    fn from(err: QueryError) -> Self {
        match err {
            QueryError::RPCError(e) => ApiError::ClientRpcError(e),
            _ => ApiError::ClientQueryError(err),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
