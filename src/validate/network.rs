use crate::api::error::{ApiError, ApiResult, UnsupportedNetworkIdentifier};
use rosetta::models::NetworkIdentifier;

#[derive(Clone)]
pub struct NetworkValidator {
    identifier: NetworkIdentifier,
}

impl NetworkValidator {
    pub fn new(identifier: NetworkIdentifier) -> Self {
        Self { identifier }
    }

    pub fn supported_networks(&self) -> Vec<NetworkIdentifier> {
        vec![self.identifier.clone()]
    }

    pub fn validate_network_identifier(&self, identifier: NetworkIdentifier) -> ApiResult<()> {
        if identifier != self.identifier {
            Err(ApiError::UnsupportedNetworkIdentifier(
                UnsupportedNetworkIdentifier::new(identifier, self.supported_networks()),
            ))
        } else {
            Ok(())
        }
    }
}
