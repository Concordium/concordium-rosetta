/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// NetworkStatusResponse : NetworkStatusResponse contains basic information about the node's view of a blockchain network. It is assumed that any BlockIdentifier.Index less than or equal to CurrentBlockIdentifier.Index can be queried.  If a Rosetta implementation prunes historical state, it should populate the optional `oldest_block_identifier` field with the oldest block available to query. If this is not populated, it is assumed that the `genesis_block_identifier` is the oldest queryable block.  If a Rosetta implementation performs some pre-sync before it is possible to query blocks, sync_status should be populated so that clients can still monitor healthiness. Without this field, it may appear that the implementation is stuck syncing and needs to be terminated. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct NetworkStatusResponse {
    #[serde(rename = "current_block_identifier")]
    pub current_block_identifier: Box<crate::models::BlockIdentifier>,
    /// The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in milliseconds because some blockchains produce blocks more often than once a second. 
    #[serde(rename = "current_block_timestamp")]
    pub current_block_timestamp: i64,
    #[serde(rename = "genesis_block_identifier")]
    pub genesis_block_identifier: Box<crate::models::BlockIdentifier>,
    #[serde(rename = "oldest_block_identifier", skip_serializing_if = "Option::is_none")]
    pub oldest_block_identifier: Option<Box<crate::models::BlockIdentifier>>,
    #[serde(rename = "sync_status", skip_serializing_if = "Option::is_none")]
    pub sync_status: Option<Box<crate::models::SyncStatus>>,
    #[serde(rename = "peers", skip_serializing_if = "Option::is_none")]
    pub peers: Option<Vec<crate::models::Peer>>,
}

impl NetworkStatusResponse {
    /// NetworkStatusResponse contains basic information about the node's view of a blockchain network. It is assumed that any BlockIdentifier.Index less than or equal to CurrentBlockIdentifier.Index can be queried.  If a Rosetta implementation prunes historical state, it should populate the optional `oldest_block_identifier` field with the oldest block available to query. If this is not populated, it is assumed that the `genesis_block_identifier` is the oldest queryable block.  If a Rosetta implementation performs some pre-sync before it is possible to query blocks, sync_status should be populated so that clients can still monitor healthiness. Without this field, it may appear that the implementation is stuck syncing and needs to be terminated. 
    pub fn new(current_block_identifier: crate::models::BlockIdentifier, current_block_timestamp: i64, genesis_block_identifier: crate::models::BlockIdentifier) -> NetworkStatusResponse {
        NetworkStatusResponse {
            current_block_identifier: Box::new(current_block_identifier),
            current_block_timestamp,
            genesis_block_identifier: Box::new(genesis_block_identifier),
            oldest_block_identifier: None,
            sync_status: None,
            peers: None,
        }
    }
}


