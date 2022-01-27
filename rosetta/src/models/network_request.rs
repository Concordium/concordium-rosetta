/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.11
 * 
 * Generated by: https://openapi-generator.tech
 */

/// NetworkRequest : A NetworkRequest is utilized to retrieve some data specific exclusively to a NetworkIdentifier. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct NetworkRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: Box<crate::models::NetworkIdentifier>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl NetworkRequest {
    /// A NetworkRequest is utilized to retrieve some data specific exclusively to a NetworkIdentifier. 
    pub fn new(network_identifier: crate::models::NetworkIdentifier) -> NetworkRequest {
        NetworkRequest {
            network_identifier: Box::new(network_identifier),
            metadata: None,
        }
    }
}


