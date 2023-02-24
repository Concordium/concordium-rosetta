/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// BlockIdentifier : The block_identifier uniquely identifies a block in a particular network. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BlockIdentifier {
    /// This is also known as the block height. 
    #[serde(rename = "index")]
    pub index: i64,
    /// This should be normalized according to the case specified in the block_hash_case network options. 
    #[serde(rename = "hash")]
    pub hash: String,
}

impl BlockIdentifier {
    /// The block_identifier uniquely identifies a block in a particular network. 
    pub fn new(index: i64, hash: String) -> BlockIdentifier {
        BlockIdentifier {
            index,
            hash,
        }
    }
}


