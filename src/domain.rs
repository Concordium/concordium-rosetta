use concordium_rust_sdk::endpoints::Client;
use rosetta::models::*;
use std::convert::Infallible;

use crate::version::*;

pub async fn network_list() -> anyhow::Result<NetworkListResponse, Infallible> {
    Ok(NetworkListResponse {
        network_identifiers: vec![NetworkIdentifier {
            blockchain: "concordium".to_string(),
            network: "mainnet".to_string(),
            sub_network_identifier: None,
        }],
    })
}

pub async fn network_options(
    _: Client,
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

// TODO Can pass Client as a mutable ref?
pub async fn network_status(
    client: Client,
    _: NetworkRequest,
) -> anyhow::Result<NetworkStatusResponse, Infallible> {
    let result = client.clone().get_consensus_status().await.unwrap();
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
