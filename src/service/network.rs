use concordium_rust_sdk::endpoints::Client;
use rosetta::models::*;
use std::convert::Infallible;

use crate::version::*;

#[derive(Clone)]
pub struct NetworkService {
    identifier: NetworkIdentifier,
    client: Client,
}

impl NetworkService {
    pub fn new(identifier: NetworkIdentifier, client: Client) -> NetworkService {
        NetworkService { identifier, client }
    }

    pub async fn network_list(&self) -> anyhow::Result<NetworkListResponse, Infallible> {
        Ok(NetworkListResponse {
            network_identifiers: vec![self.identifier.clone()],
        })
    }

    pub async fn network_options(
        &self,
        _: NetworkRequest,
    ) -> anyhow::Result<NetworkOptionsResponse, Infallible> {
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
        _: NetworkRequest,
    ) -> anyhow::Result<NetworkStatusResponse, Infallible> {
        let result = self.client.clone().get_consensus_status().await.unwrap();
        Ok(NetworkStatusResponse {
            current_block_identifier: Box::new(BlockIdentifier {
                index: result.last_finalized_block_height.height as i64,
                hash: result.last_finalized_block.to_string(),
            }),
            current_block_timestamp: result.last_finalized_time.unwrap().timestamp(),
            genesis_block_identifier: Box::new(BlockIdentifier {
                index: 0,
                hash: result.genesis_block.to_string(),
            }),
            oldest_block_identifier: None,
            sync_status: None,
            peers: vec![],
        })
    }
}
