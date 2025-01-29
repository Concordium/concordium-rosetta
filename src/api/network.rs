use crate::{
    api::{error::ApiResult, transaction::*},
    handler_error,
    validate::network::NetworkValidator,
    QueryHelper,
};
use rosetta::models::*;
use serde_json::json;

use crate::version::*;

#[derive(Clone)]
pub struct NetworkApi {
    validator:    NetworkValidator,
    query_helper: QueryHelper,
}

impl NetworkApi {
    pub fn new(validator: NetworkValidator, query_helper: QueryHelper) -> Self {
        Self {
            validator,
            query_helper,
        }
    }

    pub fn network_list(&self) -> NetworkListResponse {
        NetworkListResponse {
            network_identifiers: self.validator.supported_networks(),
        }
    }

    pub async fn network_options(&self, req: NetworkRequest) -> ApiResult<NetworkOptionsResponse> {
        self.validator.validate_network_identifier(*req.network_identifier)?;
        let node_version = self.query_helper.query_node_version().await?;
        Ok(NetworkOptionsResponse {
            version: Box::new(Version {
                rosetta_version: ROSETTA_VERSION.to_string(),
                node_version,
                middleware_version: Some(SERVER_VERSION.to_string()),
                metadata: None,
            }),
            allow:   Box::new(Allow {
                operation_statuses:        vec![
                    OperationStatus {
                        status:     OPERATION_STATUS_OK.to_string(),
                        successful: true,
                    },
                    OperationStatus {
                        status:     OPERATION_STATUS_FAIL.to_string(),
                        successful: false,
                    },
                ],
                operation_types:           vec![
                    OPERATION_TYPE_UNKNOWN.to_string(),
                    OPERATION_TYPE_FEE.to_string(),
                    OPERATION_TYPE_MINT_BAKING_REWARD.to_string(),
                    OPERATION_TYPE_MINT_FINALIZATION_REWARD.to_string(),
                    OPERATION_TYPE_MINT_PLATFORM_DEVELOPMENT_CHARGE.to_string(),
                    OPERATION_TYPE_BLOCK_REWARD.to_string(),
                    OPERATION_TYPE_BAKING_REWARD.to_string(),
                    OPERATION_TYPE_FINALIZATION_REWARD.to_string(),
                    OPERATION_TYPE_ACCOUNT_CREATION.to_string(),
                    OPERATION_TYPE_CHAIN_UPDATE.to_string(),
                    OPERATION_TYPE_DEPLOY_MODULE.to_string(),
                    OPERATION_TYPE_INIT_CONTRACT.to_string(),
                    OPERATION_TYPE_UPDATE_CONTRACT.to_string(),
                    OPERATION_TYPE_TRANSFER.to_string(),
                    OPERATION_TYPE_ADD_BAKER.to_string(),
                    OPERATION_TYPE_REMOVE_BAKER.to_string(),
                    OPERATION_TYPE_UPDATE_BAKER_STAKE.to_string(),
                    OPERATION_TYPE_UPDATE_BAKER_RESTAKE_EARNINGS.to_string(),
                    OPERATION_TYPE_UPDATE_BAKER_KEYS.to_string(),
                    OPERATION_TYPE_UPDATE_CREDENTIAL_KEYS.to_string(),
                    OPERATION_TYPE_ENCRYPTED_AMOUNT_TRANSFER.to_string(),
                    OPERATION_TYPE_TRANSFER_TO_ENCRYPTED.to_string(),
                    OPERATION_TYPE_TRANSFER_TO_PUBLIC.to_string(),
                    OPERATION_TYPE_TRANSFER_WITH_SCHEDULE.to_string(),
                    OPERATION_TYPE_UPDATE_CREDENTIALS.to_string(),
                    OPERATION_TYPE_REGISTER_DATA.to_string(),
                    OPERATION_TYPE_PAYDAY_FOUNDATION_REWARD.to_string(),
                    OPERATION_TYPE_PAYDAY_TRANSACTION_FEES_REWARD.to_string(),
                    OPERATION_TYPE_PAYDAY_BAKING_REWARD.to_string(),
                    OPERATION_TYPE_PAYDAY_FINALIZATION_REWARD.to_string(),
                    OPERATION_TYPE_BLOCK_ACCRUE_REWARD.to_string(),
                    OPERATION_TYPE_CONFIGURE_BAKER.to_string(),
                    OPERATION_TYPE_CONFIGURE_DELEGATION.to_string(),
                    OPERATION_TYPE_VALIDATOR_PRIMED_FOR_SUSPENSION.to_string(),
                    OPERATION_TYPE_VALIDATOR_SUSPENDED.to_string(),
                ],
                errors:                    vec![
                    handler_error::invalid_input_unsupported_field_error(None),
                    handler_error::invalid_input_missing_field_error(None),
                    handler_error::invalid_input_invalid_value_or_identifier_error(
                        None, None, None, None,
                    ),
                    handler_error::invalid_input_unsupported_value_error(None, None),
                    handler_error::invalid_input_inconsistent_value_error(None, None),
                    handler_error::identifier_not_resolved_no_matches_error(None),
                    handler_error::identifier_not_resolved_multiple_matches_error(None),
                    handler_error::internal_server_error(),
                    handler_error::proxy_client_rpc_error(None),
                    handler_error::proxy_client_query_error(None),
                ],
                historical_balance_lookup: true,
                timestamp_start_index:     None, /* not populated as the genesis block has a
                                                  * valid time stamp */
                call_methods:              vec![], // Call API is not implemented
                balance_exemptions:        vec![],
                mempool_coins:             false, // mempool is not available
                block_hash_case:           Some(Case::Null), // case insensitive
                transaction_hash_case:     Some(Case::Null), // case insensitive
            }),
        })
    }

    pub async fn network_status(&self, req: NetworkRequest) -> ApiResult<NetworkStatusResponse> {
        self.validator.validate_network_identifier(*req.network_identifier)?;
        let consensus_status = self.query_helper.query_consensus_info().await?;
        let peer_list = self.query_helper.client.clone().get_peers_info().await?.peers;
        Ok(NetworkStatusResponse {
            // Defining "current" block as last finalized block.
            current_block_identifier: Box::new(BlockIdentifier {
                index: consensus_status.last_finalized_block_height.height as i64,
                hash:  consensus_status.last_finalized_block.to_string(),
            }),
            current_block_timestamp:  self
                .query_helper
                .query_block_info_by_hash(&consensus_status.last_finalized_block)
                .await?
                .block_slot_time
                .timestamp_millis(),
            genesis_block_identifier: Box::new(BlockIdentifier {
                index: 0,
                hash:  consensus_status.genesis_block.to_string(),
            }),
            oldest_block_identifier:  None, /* not relevant as the implementation doesn't prune
                                             * blocks */
            sync_status:              None, /* the connected node's sync status is not easily
                                             * available and thus currently not exposed here */
            peers:                    Some(
                peer_list
                    .iter()
                    .map(|p| Peer {
                        peer_id:  p.peer_id.0.clone(),
                        metadata: Some(json!({ "ip": p.addr.ip(), "port": p.addr.port() })),
                    })
                    .collect(),
            ),
        })
    }
}
