/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.12
 * 
 * Generated by: https://openapi-generator.tech
 */

/// BlockEventType : BlockEventType determines if a BlockEvent represents the addition or removal of a block. 

/// BlockEventType determines if a BlockEvent represents the addition or removal of a block. 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum BlockEventType {
    #[serde(rename = "block_added")]
    Added,
    #[serde(rename = "block_removed")]
    Removed,

}

impl ToString for BlockEventType {
    fn to_string(&self) -> String {
        match self {
            Self::Added => String::from("block_added"),
            Self::Removed => String::from("block_removed"),
        }
    }
}

impl Default for BlockEventType {
    fn default() -> BlockEventType {
        Self::Added
    }
}




