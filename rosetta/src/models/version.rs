/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.11
 * 
 * Generated by: https://openapi-generator.tech
 */

/// Version : The Version object is utilized to inform the client of the versions of different components of the Rosetta implementation. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Version {
    /// The rosetta_version is the version of the Rosetta interface the implementation adheres to. This can be useful for clients looking to reliably parse responses. 
    #[serde(rename = "rosetta_version")]
    pub rosetta_version: String,
    /// The node_version is the canonical version of the node runtime. This can help clients manage deployments. 
    #[serde(rename = "node_version")]
    pub node_version: String,
    /// When a middleware server is used to adhere to the Rosetta interface, it should return its version here. This can help clients manage deployments. 
    #[serde(rename = "middleware_version", skip_serializing_if = "Option::is_none")]
    pub middleware_version: Option<String>,
    /// Any other information that may be useful about versioning of dependent services should be returned here. 
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Version {
    /// The Version object is utilized to inform the client of the versions of different components of the Rosetta implementation. 
    pub fn new(rosetta_version: String, node_version: String) -> Version {
        Version {
            rosetta_version,
            node_version,
            middleware_version: None,
            metadata: None,
        }
    }
}


