/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.12
 * 
 * Generated by: https://openapi-generator.tech
 */

/// ConstructionPreprocessRequest : ConstructionPreprocessRequest is passed to the `/construction/preprocess` endpoint so that a Rosetta implementation can determine which metadata it needs to request for construction.  Metadata provided in this object should NEVER be a product of live data (i.e. the caller must follow some network-specific data fetching strategy outside of the Construction API to populate required Metadata). If live data is required for construction, it MUST be fetched in the call to `/construction/metadata`.  The caller can provide a max fee they are willing to pay for a transaction. This is an array in the case fees must be paid in multiple currencies.  The caller can also provide a suggested fee multiplier to indicate that the suggested fee should be scaled. This may be used to set higher fees for urgent transactions or to pay lower fees when there is less urgency. It is assumed that providing a very low multiplier (like 0.0001) will never lead to a transaction being created with a fee less than the minimum network fee (if applicable).  In the case that the caller provides both a max fee and a suggested fee multiplier, the max fee will set an upper bound on the suggested fee (regardless of the multiplier provided). 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ConstructionPreprocessRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: Box<crate::models::NetworkIdentifier>,
    #[serde(rename = "operations")]
    pub operations: Vec<crate::models::Operation>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(rename = "max_fee", skip_serializing_if = "Option::is_none")]
    pub max_fee: Option<Vec<crate::models::Amount>>,
    #[serde(rename = "suggested_fee_multiplier", skip_serializing_if = "Option::is_none")]
    pub suggested_fee_multiplier: Option<f64>,
}

impl ConstructionPreprocessRequest {
    /// ConstructionPreprocessRequest is passed to the `/construction/preprocess` endpoint so that a Rosetta implementation can determine which metadata it needs to request for construction.  Metadata provided in this object should NEVER be a product of live data (i.e. the caller must follow some network-specific data fetching strategy outside of the Construction API to populate required Metadata). If live data is required for construction, it MUST be fetched in the call to `/construction/metadata`.  The caller can provide a max fee they are willing to pay for a transaction. This is an array in the case fees must be paid in multiple currencies.  The caller can also provide a suggested fee multiplier to indicate that the suggested fee should be scaled. This may be used to set higher fees for urgent transactions or to pay lower fees when there is less urgency. It is assumed that providing a very low multiplier (like 0.0001) will never lead to a transaction being created with a fee less than the minimum network fee (if applicable).  In the case that the caller provides both a max fee and a suggested fee multiplier, the max fee will set an upper bound on the suggested fee (regardless of the multiplier provided). 
    pub fn new(network_identifier: crate::models::NetworkIdentifier, operations: Vec<crate::models::Operation>) -> ConstructionPreprocessRequest {
        ConstructionPreprocessRequest {
            network_identifier: Box::new(network_identifier),
            operations,
            metadata: None,
            max_fee: None,
            suggested_fee_multiplier: None,
        }
    }
}


