use crate::api::{
    error::{ApiError, ApiResult, InvalidBlockIdentifierError},
    transaction::*,
};
use concordium_rust_sdk::{
    common::types::Amount,
    endpoints::{BlocksAtHeightInput, Client},
    id::types::AccountAddress,
    types::{hashes::BlockHash, queries::BlockInfo, smart_contracts::InstanceInfo, *},
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
            Address::Contract(addr) => {
                match self.client.clone().get_instance_info(addr, &block_hash).await? {
                    InstanceInfo::V0 {
                        amount,
                        ..
                    } => amount,
                    InstanceInfo::V1 {
                        amount,
                        ..
                    } => amount,
                }
            }
            Address::BakingRewardAccount => {
                match self.client.clone().get_reward_status(&block_hash).await? {
                    RewardsOverview::V0 {
                        data,
                    } => data.baking_reward_account,
                    RewardsOverview::V1 {
                        common,
                        ..
                    } => common.baking_reward_account,
                }
            }
            Address::FinalizationRewardAccount => {
                match self.client.clone().get_reward_status(&block_hash).await? {
                    RewardsOverview::V0 {
                        data,
                    } => data.finalization_reward_account,
                    RewardsOverview::V1 {
                        common,
                        ..
                    } => common.finalization_reward_account,
                }
            }
            Address::FoundationAccrueAccount => {
                match self.client.clone().get_reward_status(&block_hash).await? {
                    RewardsOverview::V0 {
                        ..
                    } => {
                        return Err(ApiError::InvalidAccountAddress(
                            ACCOUNT_ACCRUE_FOUNDATION.to_string(),
                        ))
                    }
                    RewardsOverview::V1 {
                        foundation_transaction_rewards,
                        ..
                    } => foundation_transaction_rewards,
                }
            }
            Address::PoolAccrueAccount(baker_id) => {
                match self.client.clone().get_pool_status(baker_id, &block_hash).await? {
                    PoolStatus::BakerPool {
                        current_payday_status,
                        ..
                    } => match current_payday_status {
                        None => Amount::from_ccd(0),
                        Some(s) => s.transaction_fees_earned,
                    },
                    PoolStatus::PassiveDelegation {
                        current_payday_transaction_fees_earned,
                        ..
                    } => current_payday_transaction_fees_earned,
                }
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

/// Helper type for providing a way to represent virtual reward/accrue accounts
/// in addition to ordinary ones.
pub enum Address {
    /// Real, ordinary account.
    Account(AccountAddress),
    /// Real, contract account.
    Contract(ContractAddress),
    /// Virtual baking reward account.
    BakingRewardAccount,
    /// Virtual finalization reward account.
    FinalizationRewardAccount,
    /// Virtual foundation accrue account.
    FoundationAccrueAccount,
    /// Virtual pool accrue account. Baker ID of None denotes the accrue account
    /// of the passive pool.
    PoolAccrueAccount(Option<BakerId>),
}

pub fn account_address_from_identifier(id: &AccountIdentifier) -> ApiResult<Address> {
    match id.sub_account {
        None => account_address_from_string(&id.address),
        Some(_) => Err(ApiError::SubAccountNotImplemented),
    }
}

pub fn account_address_from_string(addr: &str) -> ApiResult<Address> {
    match addr {
        ACCOUNT_REWARD_BAKING => Ok(Address::BakingRewardAccount),
        ACCOUNT_REWARD_FINALIZATION => Ok(Address::FinalizationRewardAccount),
        ACCOUNT_ACCRUE_FOUNDATION => Ok(Address::FoundationAccrueAccount),
        _ => {
            if let Some(pool) = addr.strip_prefix(ACCOUNT_ACCRUE_POOL_PREFIX) {
                if pool == POOL_PASSIVE {
                    return Ok(Address::PoolAccrueAccount(None));
                }
                let baker_id =
                    pool.parse().map_err(|_| ApiError::InvalidAccountAddress(addr.to_string()))?;
                Ok(Address::PoolAccrueAccount(Some(baker_id)))
            } else if let Some(contract_addr) = addr.strip_prefix(ACCOUNT_CONTRACT_PREFIX) {
                // TODO Improve error reporting (see parsing of signature string).
                match contract_addr.split_once('_') {
                    None => {
                        // Currently not allowing subindex to be omitted.
                        Err(ApiError::InvalidContractAddress(contract_addr.to_string()))
                    }
                    Some((contract_index, contract_subindex)) => {
                        match (contract_index.parse(), contract_subindex.parse()) {
                            (Ok(index), Ok(subindex)) => {
                                Ok(Address::Contract(ContractAddress::new(index, subindex)))
                            }
                            _ => Err(ApiError::InvalidContractAddress(contract_addr.to_string())),
                        }
                    }
                }
            } else {
                match AccountAddress::from_str(addr) {
                    Ok(a) => Ok(Address::Account(a)),
                    Err(_) => Err(ApiError::InvalidAccountAddress(addr.to_string())),
                }
            }
        }
    }
}
