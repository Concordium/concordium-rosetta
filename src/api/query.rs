use crate::api::{
    error::{ApiError, ApiResult, InvalidBlockIdentifierError},
    transaction::{ACCOUNT_BAKING_REWARD, ACCOUNT_FINALIZATION_REWARD},
};
use concordium_rust_sdk::{
    common::types::Amount,
    endpoints::{BlocksAtHeightInput, Client},
    id::types::AccountAddress,
    types::{hashes::BlockHash, queries::BlockInfo, AbsoluteBlockHeight},
};
use rosetta::models::{AccountIdentifier, PartialBlockIdentifier};
use std::str::FromStr;

#[derive(Clone)]
pub struct QueryHelper {
    pub client: Client,
}

impl QueryHelper {
    pub fn new(client: Client) -> Self {
        Self {
            client,
        }
    }

    pub async fn query_account_balance(
        &self,
        block_identifier: Option<Box<PartialBlockIdentifier>>,
        account_identifier: &AccountIdentifier,
    ) -> ApiResult<(BlockInfo, Amount)> {
        let block_info = self.query_block_info(block_identifier).await?;
        let block_hash = block_info.block_hash;
        let address = account_address_from_identifier(account_identifier)?;
        let amount = match address {
            Address::Account(addr) => {
                self.client.clone().get_account_info(addr, &block_hash).await?.account_amount
            }
            Address::BakingRewardAccount => {
                self.client.clone().get_reward_status(&block_hash).await?.baking_reward_account
            }
            Address::FinalizationRewardAccount => {
                self.client
                    .clone()
                    .get_reward_status(&block_hash)
                    .await?
                    .finalization_reward_account
            }
        };
        Ok((block_info, amount))
    }

    pub async fn query_block_info(
        &self,
        block_id: Option<Box<PartialBlockIdentifier>>,
    ) -> ApiResult<BlockInfo> {
        match block_id {
            None => {
                let consensus_status = self.client.clone().get_consensus_status().await?;
                let block_hash = consensus_status.last_finalized_block;
                Ok(self.client.clone().get_block_info(&block_hash).await?)
            }
            Some(bid) => {
                match (bid.index, bid.hash) {
                    (Some(height), None) => {
                        if height < 0 {
                            return Err(ApiError::InvalidBlockIdentifier(
                                InvalidBlockIdentifierError::InvalidIndex(height),
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
                            // (as we don't really need to return an "entire" BlockInfo, only hash
                            // and height).
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
                        InvalidBlockIdentifierError::InconsistentValues,
                    )),
                    (None, None) => {
                        Err(ApiError::InvalidBlockIdentifier(InvalidBlockIdentifierError::NoValues))
                    }
                }
            }
        }
    }
}

pub fn block_hash_from_string(hash: &str) -> ApiResult<BlockHash> {
    BlockHash::from_str(hash).map_err(|_| {
        ApiError::InvalidBlockIdentifier(InvalidBlockIdentifierError::InvalidHash(hash.to_string()))
    })
}

pub enum Address {
    Account(AccountAddress),
    BakingRewardAccount,
    FinalizationRewardAccount,
}

pub fn account_address_from_identifier(id: &AccountIdentifier) -> ApiResult<Address> {
    match id.sub_account {
        None => account_address_from_string(&id.address),
        Some(_) => Err(ApiError::SubAccountNotImplemented),
    }
}

pub fn account_address_from_string(addr: &str) -> ApiResult<Address> {
    if addr == ACCOUNT_BAKING_REWARD {
        return Ok(Address::BakingRewardAccount);
    }
    if addr == ACCOUNT_FINALIZATION_REWARD {
        return Ok(Address::FinalizationRewardAccount);
    }
    match AccountAddress::from_str(addr) {
        Ok(a) => Ok(Address::Account(a)),
        Err(_) => Err(ApiError::InvalidAccountAddress(addr.to_string())),
    }
}
