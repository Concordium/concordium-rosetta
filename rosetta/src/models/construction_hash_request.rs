/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// ConstructionHashRequest : ConstructionHashRequest is the input to the `/construction/hash` endpoint. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ConstructionHashRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: Box<crate::models::NetworkIdentifier>,
    #[serde(rename = "signed_transaction")]
    pub signed_transaction: String,
}

impl ConstructionHashRequest {
    /// ConstructionHashRequest is the input to the `/construction/hash` endpoint. 
    pub fn new(network_identifier: crate::models::NetworkIdentifier, signed_transaction: String) -> ConstructionHashRequest {
        ConstructionHashRequest {
            network_identifier: Box::new(network_identifier),
            signed_transaction,
        }
    }
}


