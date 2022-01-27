/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.11
 * 
 * Generated by: https://openapi-generator.tech
 */

/// Case : Case specifies the expected case for strings and hashes. 

/// Case specifies the expected case for strings and hashes. 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Case {
    #[serde(rename = "upper_case")]
    UpperCase,
    #[serde(rename = "lower_case")]
    LowerCase,
    #[serde(rename = "case_sensitive")]
    CaseSensitive,
    #[serde(rename = "null")]
    Null,

}

impl ToString for Case {
    fn to_string(&self) -> String {
        match self {
            Self::UpperCase => String::from("upper_case"),
            Self::LowerCase => String::from("lower_case"),
            Self::CaseSensitive => String::from("case_sensitive"),
            Self::Null => String::from("null"),
        }
    }
}

impl Default for Case {
    fn default() -> Case {
        Self::UpperCase
    }
}




