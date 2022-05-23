/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.12
 * 
 * Generated by: https://openapi-generator.tech
 */

/// OperationStatus : OperationStatus is utilized to indicate which Operation status are considered successful. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct OperationStatus {
    /// The status is the network-specific status of the operation. 
    #[serde(rename = "status")]
    pub status: String,
    /// An Operation is considered successful if the Operation.Amount should affect the Operation.Account. Some blockchains (like Bitcoin) only include successful operations in blocks but other blockchains (like Ethereum) include unsuccessful operations that incur a fee.  To reconcile the computed balance from the stream of Operations, it is critical to understand which Operation.Status indicate an Operation is successful and should affect an Account. 
    #[serde(rename = "successful")]
    pub successful: bool,
}

impl OperationStatus {
    /// OperationStatus is utilized to indicate which Operation status are considered successful. 
    pub fn new(status: String, successful: bool) -> OperationStatus {
        OperationStatus {
            status,
            successful,
        }
    }
}


