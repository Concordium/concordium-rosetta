use crate::api::error::{ApiError, ApiResult};
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
            return Err(ApiError::UnsupportedNetworkIdentifier);
        }
        Ok(())
    }
}
