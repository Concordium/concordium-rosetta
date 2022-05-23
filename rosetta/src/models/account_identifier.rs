/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.12
 * 
 * Generated by: https://openapi-generator.tech
 */

/// AccountIdentifier : The account_identifier uniquely identifies an account within a network. All fields in the account_identifier are utilized to determine this uniqueness (including the metadata field, if populated). 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AccountIdentifier {
    /// The address may be a cryptographic public key (or some encoding of it) or a provided username. 
    #[serde(rename = "address")]
    pub address: String,
    #[serde(rename = "sub_account", skip_serializing_if = "Option::is_none")]
    pub sub_account: Option<Box<crate::models::SubAccountIdentifier>>,
    /// Blockchains that utilize a username model (where the address is not a derivative of a cryptographic public key) should specify the public key(s) owned by the address in metadata. 
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl AccountIdentifier {
    /// The account_identifier uniquely identifies an account within a network. All fields in the account_identifier are utilized to determine this uniqueness (including the metadata field, if populated). 
    pub fn new(address: String) -> AccountIdentifier {
        AccountIdentifier {
            address,
            sub_account: None,
            metadata: None,
        }
    }
}


