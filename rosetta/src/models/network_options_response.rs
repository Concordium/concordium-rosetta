/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.11
 * 
 * Generated by: https://openapi-generator.tech
 */

/// NetworkOptionsResponse : NetworkOptionsResponse contains information about the versioning of the node and the allowed operation statuses, operation types, and errors. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct NetworkOptionsResponse {
    #[serde(rename = "version")]
    pub version: Box<crate::models::Version>,
    #[serde(rename = "allow")]
    pub allow: Box<crate::models::Allow>,
}

impl NetworkOptionsResponse {
    /// NetworkOptionsResponse contains information about the versioning of the node and the allowed operation statuses, operation types, and errors. 
    pub fn new(version: crate::models::Version, allow: crate::models::Allow) -> NetworkOptionsResponse {
        NetworkOptionsResponse {
            version: Box::new(version),
            allow: Box::new(allow),
        }
    }
}


