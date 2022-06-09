use concordium_rust_sdk::endpoints::{QueryError, RPCError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvalidBlockIdentifierError {
    #[error("no values")]
    NoValues,
    #[error("inconsistent values: hash and index are mutually exclusive")]
    InconsistentValues,
    #[error("invalid hash value '{0}'")]
    InvalidHash(String),
    #[error("invalid index value '{0}'")]
    InvalidIndex(i64),
}

#[derive(Error, Debug)]
pub enum InvalidSignatureError {
    #[error("separator '{0}' is missing")]
    MissingSeparator(String),
    #[error("index separator '{0}' is missing")]
    MissingIndexSeparator(String),
    #[error("invalid credential index '{0}'")]
    InvalidCredentialIndex(String),
    #[error("invalid key index '{0}'")]
    InvalidKeyIndex(String),
    #[error("invalid signature hex bytes '{0}'")]
    InvalidSignatureHexBytes(String),
}

#[derive(Error, Debug)]
pub enum ApiError {
    // Invalid input: Unsupported field.
    #[error("field '{0}' is not supported")]
    UnsupportedFieldPresent(String),
    #[error("sub-accounts are not supported")]
    SubAccountNotImplemented,

    // Invalid input: Missing field.
    #[error("required field '{0}' is missing")]
    RequiredFieldMissing(String),

    // Invalid input: Invalid value or identifier (type or format).
    #[error("invalid account address '{0}'")]
    InvalidAccountAddress(String),
    #[error("invalid contract address '{0}'")]
    InvalidContractAddress(String),
    #[error("invalid currency")]
    InvalidCurrency,
    #[error("invalid amount '{0}'")]
    InvalidAmount(String),
    #[error("invalid block identifier")]
    InvalidBlockIdentifier(InvalidBlockIdentifierError),
    #[error("invalid signature '{0}': {1}")]
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

    // Invalid input: Unsupported field value.
    #[error("unsupported operation type '{0}'")]
    UnsupportedOperationType(String),

    // Invalid input: Inconsistent value.
    #[error("inconsistent operations: {0}")]
    InconsistentOperations(String),

    // Identifier not resolved: Unresolved identifier.
    #[error("unsupported network identifier provided")]
    UnsupportedNetworkIdentifier,
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
    fn from(err: RPCError) -> Self { ApiError::ClientRpcError(err) }
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
