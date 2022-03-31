use crate::api::error::{ApiError, ApiResult, InvalidBlockIdentifier};
use concordium_rust_sdk::endpoints::{BlocksAtHeightInput, Client};
use concordium_rust_sdk::id::types::AccountAddress;
use concordium_rust_sdk::types::hashes::BlockHash;
use concordium_rust_sdk::types::queries::BlockInfo;
use concordium_rust_sdk::types::{AbsoluteBlockHeight, AccountInfo};
use rosetta::models::{AccountIdentifier, PartialBlockIdentifier};
use std::str::FromStr;

#[derive(Clone)]
pub struct QueryHelper {
    pub client: Client,
}

impl QueryHelper {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn query_account_info(
        &self,
        block_identifier: Option<Box<PartialBlockIdentifier>>,
        account_identifier: &AccountIdentifier,
    ) -> ApiResult<(BlockInfo, AccountInfo)> {
        let block_info = self.query_block_info(block_identifier).await?;
        let block_hash = block_hash_from_string(block_info.block_hash.to_string().as_str())?;
        let address = account_address_from_identifier(account_identifier)?;
        Ok((
            block_info,
            self.client
                .clone()
                .get_account_info(address, &block_hash)
                .await?,
        ))
    }

    pub async fn query_block_info(
        &self,
        block_id: Option<Box<PartialBlockIdentifier>>,
    ) -> ApiResult<BlockInfo> {
        match block_id {
            None => {
                let consensus_status = self.client.clone().get_consensus_status().await?;
                let block_hash = block_hash_from_string(
                    consensus_status.last_finalized_block.to_string().as_str(),
                )?;
                Ok(self.client.clone().get_block_info(&block_hash).await?)
            }
            Some(bid) => {
                match (bid.index, bid.hash) {
                    (Some(height), None) => {
                        if height < 0 {
                            return Err(ApiError::InvalidBlockIdentifier(
                                InvalidBlockIdentifier::InvalidIndex,
                            ));
                        }
                        let blocks = self
                            .client
                            .clone()
                            .get_blocks_at_height(BlocksAtHeightInput::Absolute {
                                height: AbsoluteBlockHeight {
                                    height: height as u64,
                                },
                            })
                            .await?;
                        match blocks[..] {
                            [] => Err(ApiError::NoBlocksMatched),
                            // Note that unless we decide to return additional block metadata,
                            // this particular GetBlockInfo call is redundant
                            // (as we don't really need to return an "entire" BlockInfo, only hash and height).
                            [block] => Ok(self.client.clone().get_block_info(&block).await?),
                            _ => Err(ApiError::MultipleBlocksMatched),
                        }
                    }
                    (None, Some(hash)) => {
                        let block_hash = block_hash_from_string(hash.as_str())?;
                        Ok(self.client.clone().get_block_info(&block_hash).await?)
                    }
                    // TODO Allow if height and hash are consistent.
                    (Some(_), Some(_)) => Err(ApiError::InvalidBlockIdentifier(
                        InvalidBlockIdentifier::InconsistentValues,
                    )),
                    (None, None) => Err(ApiError::InvalidBlockIdentifier(
                        InvalidBlockIdentifier::NoValues,
                    )),
                }
            }
        }
    }
}

pub fn block_hash_from_string(hash: &str) -> ApiResult<BlockHash> {
    BlockHash::from_str(hash)
        .map_err(|_| ApiError::InvalidBlockIdentifier(InvalidBlockIdentifier::InvalidHash))
}

pub fn account_address_from_identifier(id: &AccountIdentifier) -> ApiResult<AccountAddress> {
    match id.sub_account {
        None => account_address_from_string(&id.address),
        Some(_) => Err(ApiError::SubAccountNotImplemented),
    }
}

pub fn account_address_from_string(addr: &String) -> ApiResult<AccountAddress> {
    AccountAddress::from_str(addr.as_str())
        .map_err(|_| ApiError::InvalidAccountAddress(addr.clone()))
}
