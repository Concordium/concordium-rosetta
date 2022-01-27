/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.11
 * 
 * Generated by: https://openapi-generator.tech
 */

/// Amount : Amount is some Value of a Currency. It is considered invalid to specify a Value without a Currency. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Amount {
    /// Value of the transaction in atomic units represented as an arbitrary-sized signed integer.  For example, 1 BTC would be represented by a value of 100000000. 
    #[serde(rename = "value")]
    pub value: String,
    #[serde(rename = "currency")]
    pub currency: Box<crate::models::Currency>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Amount {
    /// Amount is some Value of a Currency. It is considered invalid to specify a Value without a Currency. 
    pub fn new(value: String, currency: crate::models::Currency) -> Amount {
        Amount {
            value,
            currency: Box::new(currency),
            metadata: None,
        }
    }
}


