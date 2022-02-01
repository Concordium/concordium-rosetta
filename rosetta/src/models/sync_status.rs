/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.10
 * 
 * Generated by: https://openapi-generator.tech
 */

/// SyncStatus : SyncStatus is used to provide additional context about an implementation's sync status.  This object is often used by implementations to indicate healthiness when block data cannot be queried until some sync phase completes or cannot be determined by comparing the timestamp of the most recent block with the current time. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SyncStatus {
    /// CurrentIndex is the index of the last synced block in the current stage.  This is a separate field from current_block_identifier in NetworkStatusResponse because blocks with indices up to and including the current_index may not yet be queryable by the caller. To reiterate, all indices up to and including current_block_identifier in NetworkStatusResponse must be queryable via the /block endpoint (excluding indices less than oldest_block_identifier). 
    #[serde(rename = "current_index", skip_serializing_if = "Option::is_none")]
    pub current_index: Option<i64>,
    /// TargetIndex is the index of the block that the implementation is attempting to sync to in the current stage. 
    #[serde(rename = "target_index", skip_serializing_if = "Option::is_none")]
    pub target_index: Option<i64>,
    /// Stage is the phase of the sync process. 
    #[serde(rename = "stage", skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    /// sycned is a boolean that indicates if an implementation has synced up to the most recent block. If this field is not populated, the caller should rely on a traditional tip timestamp comparison to determine if an implementation is synced.  This field is particularly useful for quiescent blockchains (blocks only produced when there are pending transactions). In these blockchains, the most recent block could have a timestamp far behind the current time but the node could be healthy and at tip. 
    #[serde(rename = "synced", skip_serializing_if = "Option::is_none")]
    pub synced: Option<bool>,
}

impl SyncStatus {
    /// SyncStatus is used to provide additional context about an implementation's sync status.  This object is often used by implementations to indicate healthiness when block data cannot be queried until some sync phase completes or cannot be determined by comparing the timestamp of the most recent block with the current time. 
    pub fn new() -> SyncStatus {
        SyncStatus {
            current_index: None,
            target_index: None,
            stage: None,
            synced: None,
        }
    }
}


