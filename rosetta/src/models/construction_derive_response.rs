/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.10
 * 
 * Generated by: https://openapi-generator.tech
 */

/// ConstructionDeriveResponse : ConstructionDeriveResponse is returned by the `/construction/derive` endpoint. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ConstructionDeriveResponse {
    /// [DEPRECATED by `account_identifier` in `v1.4.4`] Address in network-specific format. 
    #[serde(rename = "address", skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(rename = "account_identifier", skip_serializing_if = "Option::is_none")]
    pub account_identifier: Option<Box<crate::models::AccountIdentifier>>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl ConstructionDeriveResponse {
    /// ConstructionDeriveResponse is returned by the `/construction/derive` endpoint. 
    pub fn new() -> ConstructionDeriveResponse {
        ConstructionDeriveResponse {
            address: None,
            account_identifier: None,
            metadata: None,
        }
    }
}


