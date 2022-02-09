use crate::api::network::Error::ClientRpcError;
use concordium_rust_sdk::endpoints::{Client, RPCError};
use rosetta::models::*;
use std::ops::Deref;
use thiserror::Error;

use crate::version::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unsupported network identifier provided")]
    UnsupportedNetworkIdentifier,
    #[error("client RPC error")]
    ClientRpcError(RPCError),
}

impl warp::reject::Reject for Error {}

impl From<concordium_rust_sdk::endpoints::RPCError> for Error {
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

    pub async fn network_list(&self) -> Result<NetworkListResponse, Error> {
        Ok(NetworkListResponse {
            network_identifiers: vec![self.identifier.clone()],
        })
    }

    pub async fn network_options(
        &self,
        req: NetworkRequest,
    ) -> Result<NetworkOptionsResponse, Error> {
        if req.network_identifier.deref() != &self.identifier {
            return Err(Error::UnsupportedNetworkIdentifier);
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
    ) -> Result<NetworkStatusResponse, Error> {
        if req.network_identifier.deref() != &self.identifier {
            return Err(Error::UnsupportedNetworkIdentifier);
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
