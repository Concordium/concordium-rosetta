/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// MempoolTransactionResponse : A MempoolTransactionResponse contains an estimate of a mempool transaction. It may not be possible to know the full impact of a transaction in the mempool (ex: fee paid). 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MempoolTransactionResponse {
    #[serde(rename = "transaction")]
    pub transaction: Box<crate::models::Transaction>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl MempoolTransactionResponse {
    /// A MempoolTransactionResponse contains an estimate of a mempool transaction. It may not be possible to know the full impact of a transaction in the mempool (ex: fee paid). 
    pub fn new(transaction: crate::models::Transaction) -> MempoolTransactionResponse {
        MempoolTransactionResponse {
            transaction: Box::new(transaction),
            metadata: None,
        }
    }
}


