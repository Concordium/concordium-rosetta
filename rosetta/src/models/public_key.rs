/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.11
 * 
 * Generated by: https://openapi-generator.tech
 */

/// PublicKey : PublicKey contains a public key byte array for a particular CurveType encoded in hex.  Note that there is no PrivateKey struct as this is NEVER the concern of an implementation. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PublicKey {
    /// Hex-encoded public key bytes in the format specified by the CurveType. 
    #[serde(rename = "hex_bytes")]
    pub hex_bytes: String,
    #[serde(rename = "curve_type")]
    pub curve_type: crate::models::CurveType,
}

impl PublicKey {
    /// PublicKey contains a public key byte array for a particular CurveType encoded in hex.  Note that there is no PrivateKey struct as this is NEVER the concern of an implementation. 
    pub fn new(hex_bytes: String, curve_type: crate::models::CurveType) -> PublicKey {
        PublicKey {
            hex_bytes,
            curve_type,
        }
    }
}


