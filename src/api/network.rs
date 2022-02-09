use crate::api::network::ApiError::ClientRpcError;
use concordium_rust_sdk::endpoints::{Client, RPCError};
use rosetta::models::*;
use serde::Serialize;
use std::ops::Deref;
use thiserror::Error;

use crate::version::*;

#[derive(Debug, Serialize)]
pub struct UnsupportedNetworkIdentifier {
    provided: NetworkIdentifier,
    supported: Vec<NetworkIdentifier>,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("unsupported network identifier provided")]
    UnsupportedNetworkIdentifier(UnsupportedNetworkIdentifier),
    #[error("client RPC error")]
    ClientRpcError(RPCError),
}

impl warp::reject::Reject for ApiError {}

impl From<concordium_rust_sdk::endpoints::RPCError> for ApiError {
    fn from(err: RPCError) -> Self {
        ClientRpcError(err)
    }
}

#[derive(Clone)]
pub struct NetworkApi {
    identifier: NetworkIdentifier,
    client: Client,
}

impl NetworkApi {
    pub fn new(identifier: NetworkIdentifier, client: Client) -> Self {
        NetworkApi { identifier, client }
    }

    pub fn network_list(&self) -> NetworkListResponse {
        NetworkListResponse {
            network_identifiers: vec![self.identifier.clone()],
        }
    }

    pub async fn network_options(
        &self,
        req: NetworkRequest,
    ) -> Result<NetworkOptionsResponse, ApiError> {
        if req.network_identifier.deref() != &self.identifier {
            return Err(ApiError::UnsupportedNetworkIdentifier(
                UnsupportedNetworkIdentifier {
                    provided: *req.network_identifier,
                    supported: self.network_list().network_identifiers,
                },
            ));
        }
        Ok(NetworkOptionsResponse {
            version: Box::new(Version {
                rosetta_version: ROSETTA_VERSION.to_string(),
                node_version: NODE_VERSION.to_string(),
                middleware_version: Some(SERVER_VERSION.to_string()),
                metadata: None,
            }),
            allow: Box::new(Default::default()),
        })
    }

    pub async fn network_status(
        &self,
        req: NetworkRequest,
    ) -> Result<NetworkStatusResponse, ApiError> {
        if req.network_identifier.deref() != &self.identifier {
            return Err(ApiError::UnsupportedNetworkIdentifier(
                UnsupportedNetworkIdentifier {
                    provided: *req.network_identifier,
                    supported: self.network_list().network_identifiers,
                },
            ));
        }
        let consensus_status = self.client.clone().get_consensus_status().await?;
        Ok(NetworkStatusResponse {
            current_block_identifier: Box::new(BlockIdentifier {
                index: consensus_status.last_finalized_block_height.height as i64,
                hash: consensus_status.last_finalized_block.to_string(),
            }),
            current_block_timestamp: consensus_status.last_finalized_time.unwrap().timestamp(),
            genesis_block_identifier: Box::new(BlockIdentifier {
                index: 0,
                hash: consensus_status.genesis_block.to_string(),
            }),
            oldest_block_identifier: None,
            sync_status: None,
            peers: vec![],
        })
    }
}
