use crate::api::network::ApiError::ClientRpcError;
use concordium_rust_sdk::endpoints::{Client, RPCError};
use rosetta::models::*;
use serde::Serialize;
use serde_json::json;
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
            allow: Box::new(Allow {
                operation_statuses: vec![],       // none yet
                operation_types: vec![],          // none yet
                errors: vec![], // TODO should be one result for each known error code?
                historical_balance_lookup: false, // TODO do we support querying account balance at any block height?
                timestamp_start_index: None, // not populated as the genesis block has a valid time stamp
                call_methods: vec![],        // none yet
                balance_exemptions: vec![], // TODO unsure what this is for - rewards are paid out without operations??
                mempool_coins: false,       // mempool is not available
            }),
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
        let peer_list = self.client.clone().peer_list(false).await?;
        Ok(NetworkStatusResponse {
            // Defining "current" block as last finalized block.
            current_block_identifier: Box::new(BlockIdentifier {
                index: consensus_status.last_finalized_block_height.height as i64,
                hash: consensus_status.last_finalized_block.to_string(),
            }),
            current_block_timestamp: consensus_status
                .last_finalized_time
                .map(|t| t.timestamp_millis())
                .unwrap_or(-1),
            genesis_block_identifier: Box::new(BlockIdentifier {
                index: 0,
                hash: consensus_status.genesis_block.to_string(),
            }),
            oldest_block_identifier: None, // not relevant as the implementation doesn't prune blocks
            sync_status: None, // the connected node's sync status is not easily available and thus currently not exposed here
            peers: peer_list
                .iter()
                .map(|p| Peer {
                    peer_id: p.node_id.to_string(),
                    metadata: Some(json!({ "ip": p.ip, "port": p.port })),
                })
                .collect(),
        })
    }
}
