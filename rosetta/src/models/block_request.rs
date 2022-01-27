/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.11
 * 
 * Generated by: https://openapi-generator.tech
 */

/// BlockRequest : A BlockRequest is utilized to make a block request on the /block endpoint. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BlockRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: Box<crate::models::NetworkIdentifier>,
    #[serde(rename = "block_identifier")]
    pub block_identifier: Box<crate::models::PartialBlockIdentifier>,
}

impl BlockRequest {
    /// A BlockRequest is utilized to make a block request on the /block endpoint. 
    pub fn new(network_identifier: crate::models::NetworkIdentifier, block_identifier: crate::models::PartialBlockIdentifier) -> BlockRequest {
        BlockRequest {
            network_identifier: Box::new(network_identifier),
            block_identifier: Box::new(block_identifier),
        }
    }
}


