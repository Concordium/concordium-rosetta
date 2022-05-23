/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.12
 * 
 * Generated by: https://openapi-generator.tech
 */

/// Error : Instead of utilizing HTTP status codes to describe node errors (which often do not have a good analog), rich errors are returned using this object.  Both the code and message fields can be individually used to correctly identify an error. Implementations MUST use unique values for both fields. 



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Error {
    /// Code is a network-specific error code. If desired, this code can be equivalent to an HTTP status code. 
    #[serde(rename = "code")]
    pub code: i32,
    /// Message is a network-specific error message.  The message MUST NOT change for a given code. In particular, this means that any contextual information should be included in the details field. 
    #[serde(rename = "message")]
    pub message: String,
    /// Description allows the implementer to optionally provide additional information about an error. In many cases, the content of this field will be a copy-and-paste from existing developer documentation.  Description can ONLY be populated with generic information about a particular type of error. It MUST NOT be populated with information about a particular instantiation of an error (use `details` for this).  Whereas the content of Error.Message should stay stable across releases, the content of Error.Description will likely change across releases (as implementers improve error documentation). For this reason, the content in this field is not part of any type assertion (unlike Error.Message). 
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// An error is retriable if the same request may succeed if submitted again. 
    #[serde(rename = "retriable")]
    pub retriable: bool,
    /// Often times it is useful to return context specific to the request that caused the error (i.e. a sample of the stack trace or impacted account) in addition to the standard error message. 
    #[serde(rename = "details", skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl Error {
    /// Instead of utilizing HTTP status codes to describe node errors (which often do not have a good analog), rich errors are returned using this object.  Both the code and message fields can be individually used to correctly identify an error. Implementations MUST use unique values for both fields. 
    pub fn new(code: i32, message: String, retriable: bool) -> Error {
        Error {
            code,
            message,
            description: None,
            retriable,
            details: None,
        }
    }
}


