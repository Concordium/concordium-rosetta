/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.12
 * 
 * Generated by: https://openapi-generator.tech
 */

/// Allow : Allow specifies supported Operation status, Operation types, and all possible error statuses. This Allow object is used by clients to validate the correctness of a Rosetta Server implementation. It is expected that these clients will error if they receive some response that contains any of the above information that is not specified here. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Allow {
    /// All Operation.Status this implementation supports. Any status that is returned during parsing that is not listed here will cause client validation to error. 
    #[serde(rename = "operation_statuses")]
    pub operation_statuses: Vec<crate::models::OperationStatus>,
    /// All Operation.Type this implementation supports. Any type that is returned during parsing that is not listed here will cause client validation to error. 
    #[serde(rename = "operation_types")]
    pub operation_types: Vec<String>,
    /// All Errors that this implementation could return. Any error that is returned during parsing that is not listed here will cause client validation to error. 
    #[serde(rename = "errors")]
    pub errors: Vec<crate::models::Error>,
    /// Any Rosetta implementation that supports querying the balance of an account at any height in the past should set this to true. 
    #[serde(rename = "historical_balance_lookup")]
    pub historical_balance_lookup: bool,
    /// If populated, `timestamp_start_index` indicates the first block index where block timestamps are considered valid (i.e. all blocks less than `timestamp_start_index` could have invalid timestamps). This is useful when the genesis block (or blocks) of a network have timestamp 0.  If not populated, block timestamps are assumed to be valid for all available blocks. 
    #[serde(rename = "timestamp_start_index", skip_serializing_if = "Option::is_none")]
    pub timestamp_start_index: Option<i64>,
    /// All methods that are supported by the /call endpoint. Communicating which parameters should be provided to /call is the responsibility of the implementer (this is en lieu of defining an entire type system and requiring the implementer to define that in Allow). 
    #[serde(rename = "call_methods")]
    pub call_methods: Vec<String>,
    /// BalanceExemptions is an array of BalanceExemption indicating which account balances could change without a corresponding Operation.  BalanceExemptions should be used sparingly as they may introduce significant complexity for integrators that attempt to reconcile all account balance changes.  If your implementation relies on any BalanceExemptions, you MUST implement historical balance lookup (the ability to query an account balance at any BlockIdentifier). 
    #[serde(rename = "balance_exemptions")]
    pub balance_exemptions: Vec<crate::models::BalanceExemption>,
    /// Any Rosetta implementation that can update an AccountIdentifier's unspent coins based on the contents of the mempool should populate this field as true. If false, requests to `/account/coins` that set `include_mempool` as true will be automatically rejected. 
    #[serde(rename = "mempool_coins")]
    pub mempool_coins: bool,
    #[serde(rename = "block_hash_case", skip_serializing_if = "Option::is_none")]
    pub block_hash_case: Option<crate::models::Case>,
    #[serde(rename = "transaction_hash_case", skip_serializing_if = "Option::is_none")]
    pub transaction_hash_case: Option<crate::models::Case>,
}

impl Allow {
    /// Allow specifies supported Operation status, Operation types, and all possible error statuses. This Allow object is used by clients to validate the correctness of a Rosetta Server implementation. It is expected that these clients will error if they receive some response that contains any of the above information that is not specified here. 
    pub fn new(operation_statuses: Vec<crate::models::OperationStatus>, operation_types: Vec<String>, errors: Vec<crate::models::Error>, historical_balance_lookup: bool, call_methods: Vec<String>, balance_exemptions: Vec<crate::models::BalanceExemption>, mempool_coins: bool) -> Allow {
        Allow {
            operation_statuses,
            operation_types,
            errors,
            historical_balance_lookup,
            timestamp_start_index: None,
            call_methods,
            balance_exemptions,
            mempool_coins,
            block_hash_case: None,
            transaction_hash_case: None,
        }
    }
}


