/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.12
 * 
 * Generated by: https://openapi-generator.tech
 */

/// BlockTransactionRequest : A BlockTransactionRequest is used to fetch a Transaction included in a block that is not returned in a BlockResponse. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BlockTransactionRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: Box<crate::models::NetworkIdentifier>,
    #[serde(rename = "block_identifier")]
    pub block_identifier: Box<crate::models::BlockIdentifier>,
    #[serde(rename = "transaction_identifier")]
    pub transaction_identifier: Box<crate::models::TransactionIdentifier>,
}

impl BlockTransactionRequest {
    /// A BlockTransactionRequest is used to fetch a Transaction included in a block that is not returned in a BlockResponse. 
    pub fn new(network_identifier: crate::models::NetworkIdentifier, block_identifier: crate::models::BlockIdentifier, transaction_identifier: crate::models::TransactionIdentifier) -> BlockTransactionRequest {
        BlockTransactionRequest {
            network_identifier: Box::new(network_identifier),
            block_identifier: Box::new(block_identifier),
            transaction_identifier: Box::new(transaction_identifier),
        }
    }
}


