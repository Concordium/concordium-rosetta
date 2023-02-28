use crate::api::{
    error::{ApiError, ApiResult, InvalidBlockIdentifierError},
    transaction::*,
};
use concordium_rust_sdk::{
    common::types::Amount,
    endpoints::{BlocksAtHeightInput, Client, QueryError},
    id::types::AccountAddress,
    types::{hashes::BlockHash, queries::BlockInfo, smart_contracts::InstanceInfo, *}, v2,
};
use rosetta::models::{AccountIdentifier, PartialBlockIdentifier};
use std::str::FromStr;

#[derive(Clone)]
// TODO: After all translation is done, remove this:
pub struct QueryHelper {
    pub client: Client,
}

#[derive(Clone)]
pub struct QueryHelperV2 {
    pub client: v2::Client,
}


impl QueryHelperV2 {
    pub fn new(client: v2::Client) -> Self {
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
                let accId = v2::AccountIdentifier::Address(addr);
                match self.client.clone().get_account_info(&accId, &block_hash).await {
                    Ok(i) => i.response.account_amount,
                    Err(err) => match err {
                        QueryError::RPCError(err) => return Err(err.into()),
                        QueryError::NotFound => Amount::from_micro_ccd(0),
                    },
                }
            }
            Address::Contract(addr) => {
                match self.client.clone().get_instance_info(addr, &block_hash).await {
                    Ok(i) => match i.response {
                        InstanceInfo::V0 {
                            amount,
                            ..
                        } => amount,
                        InstanceInfo::V1 {
                            amount,
                            ..
                        } => amount,
                    },
                    Err(err) => match err {
                        QueryError::RPCError(err) => return Err(err.into()),
                        QueryError::NotFound => Amount::from_micro_ccd(0),
                    },
                }
            }
            Address::BakingRewardAccount => {
                panic!("");
            }
            Address::FinalizationRewardAccount => {
                panic!("");
            }
            Address::FoundationAccrueAccount => {
                panic!("");
            }
            Address::PoolAccrueAccount(baker_id) => {
                let baker_id_unwrapped = baker_id.unwrap();
                match self.client.clone().get_pool_info(&block_hash, baker_id_unwrapped).await {
                    Ok(i) => {
                        // TODO: fix this, it's weird
                        let x = match i.response.current_payday_status {
                            None => Amount::from_ccd(0),
                            Some(s) => s.transaction_fees_earned,
                        };
                        x
                    },
                    Err(err) => match err {
                        QueryError::RPCError(err) => return Err(err.into()),
                        QueryError::NotFound => Amount::from_micro_ccd(0),
                    },
                }
            }
        };
        Ok((block_info, amount))
    }

    pub async fn query_account_info_by_address(
        &self,
        addr: AccountAddress,
        block_hash: &BlockHash,
    ) -> ApiResult<AccountInfo> {
        let acc_id = v2::AccountIdentifier::Address(addr);
        Ok(self.client.clone().get_account_info(&acc_id, block_hash).await.map_err(|_| ApiError::InvalidCurrency)?.response)
    }

    pub async fn query_block_info_by_hash(&self, block_hash: impl v2::IntoBlockIdentifier) -> ApiResult<BlockInfo> {
        Ok(self.client.clone().get_block_info(block_hash).await.map_err(|_| ApiError::InvalidCurrency)?.response)
    }

    /*
    pub async fn query_block_summary_by_hash(
        &self,
        block_hash: &BlockHash,
    ) -> ApiResult<BlockSummary> {

        match self.client.clone().get_block_transaction_events(block_hash).await {
            Ok(o) => {
                let x: Vec<(BlockItemSummary, )> = o.response.into_stream().try_collect();
      kk      }
            Err(e) => Err(ApiError::ClientQueryError(e)),
        }
    }
    */

    /*
    pub async fn query_reward_status_by_hash(
        &self,
        block_hash: &BlockHash,
    ) -> ApiResult<RewardsOverview> {
        map_query_result(
            self.client.clone().get_reward_status(block_hash).await,
            ApiError::NoBlocksMatched,
        )
    }
    */

    pub async fn query_block_hash_from_height(&self, height: i64) -> ApiResult<BlockHash> {
        if height < 0 {
            return Err(ApiError::InvalidBlockIdentifier(
                InvalidBlockIdentifierError::InvalidIndex(height),
            ));
        };
        let block_height = BlocksAtHeightInput::Absolute {
            height: AbsoluteBlockHeight {
                height: height as u64,
            },
        };
        let blocks = self
            .client
            .clone()
            .get_blocks_at_height(&block_height)
            .await
            .map_err(|_| ApiError::InvalidCurrency)?;
        match blocks[..] {
            [] => Err(ApiError::NoBlocksMatched),
            // Note that unless we decide to return additional block metadata,
            // this particular GetBlockInfo call is redundant
            // (as we don't really need to return an "entire" BlockInfo, only hash
            // and height).
            [block_hash] => Ok(block_hash),
            _ => Err(ApiError::MultipleBlocksMatched),
        }
    }

    pub async fn query_block_info(
        &self,
        block_id: Option<Box<PartialBlockIdentifier>>,
    ) -> ApiResult<BlockInfo> {
        match block_id {
            None => {
                self.query_block_info_by_hash(v2::BlockIdentifier::LastFinal).await
            }
            Some(bid) => {
                match (bid.index, bid.hash) {
                    (Some(height), None) => {
                        let block_hash = self.query_block_hash_from_height(height).await?;
                        self.query_block_info_by_hash(block_hash).await
                    }
                    (None, Some(hash)) => {
                        let block_hash = block_hash_from_string(hash.as_str())?;
                        self.query_block_info_by_hash(&block_hash).await
                    }
                    (Some(height), Some(hash)) => {
                        let block_hash_string = block_hash_from_string(hash.as_str())?;
                        let block_hash_height = self.query_block_hash_from_height(height).await?;

                        if block_hash_string == block_hash_height {
                            self.query_block_info_by_hash(&block_hash_string).await
                        } else {
                            Err(ApiError::InvalidBlockIdentifier(InvalidBlockIdentifierError::InconsistentValues))
                        }
                    }
                    (None, None) => {
                        Err(ApiError::InvalidBlockIdentifier(InvalidBlockIdentifierError::NoValues))
                    }
                }
            }
        }
    }
}

// TODO: After all translation is done, remove this:
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
                match self.client.clone().get_account_info(addr, &block_hash).await {
                    Ok(i) => i.account_amount,
                    Err(err) => match err {
                        QueryError::RPCError(err) => return Err(err.into()),
                        QueryError::NotFound => Amount::from_micro_ccd(0),
                    },
                }
            }
            Address::Contract(addr) => {
                match self.client.clone().get_instance_info(addr, &block_hash).await {
                    Ok(i) => match i {
                        InstanceInfo::V0 {
                            amount,
                            ..
                        } => amount,
                        InstanceInfo::V1 {
                            amount,
                            ..
                        } => amount,
                    },
                    Err(err) => match err {
                        QueryError::RPCError(err) => return Err(err.into()),
                        QueryError::NotFound => Amount::from_micro_ccd(0),
                    },
                }
            }
            Address::BakingRewardAccount => {
                match self.query_reward_status_by_hash(&block_hash).await? {
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
                match self.query_reward_status_by_hash(&block_hash).await? {
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
                match self.query_reward_status_by_hash(&block_hash).await? {
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
                match self.client.clone().get_pool_status(baker_id, &block_hash).await {
                    Ok(i) => match i {
                        PoolStatus::BakerPool {
                            status,
                        } => match status.current_payday_status {
                            None => Amount::from_ccd(0),
                            Some(s) => s.transaction_fees_earned,
                        },
                        PoolStatus::PassiveDelegation {
                            status,
                        } => status.current_payday_transaction_fees_earned,
                    },
                    Err(err) => match err {
                        QueryError::RPCError(err) => return Err(err.into()),
                        QueryError::NotFound => Amount::from_micro_ccd(0),
                    },
                }
            }
        };
        Ok((block_info, amount))
    }

    pub async fn query_account_info_by_address(
        &self,
        addr: AccountAddress,
        block_hash: &BlockHash,
    ) -> ApiResult<AccountInfo> {
        map_query_result(
            self.client.clone().get_account_info(addr, block_hash).await,
            ApiError::NoAccountsMatched,
        )
    }

    pub async fn query_block_info_by_hash(&self, block_hash: &BlockHash) -> ApiResult<BlockInfo> {
        map_query_result(
            self.client.clone().get_block_info(block_hash).await,
            ApiError::NoBlocksMatched,
        )
    }

    pub async fn query_block_summary_by_hash(
        &self,
        block_hash: &BlockHash,
    ) -> ApiResult<BlockSummary> {
        map_query_result(
            self.client.clone().get_block_summary(block_hash).await,
            ApiError::NoBlocksMatched,
        )
    }

    pub async fn query_reward_status_by_hash(
        &self,
        block_hash: &BlockHash,
    ) -> ApiResult<RewardsOverview> {
        map_query_result(
            self.client.clone().get_reward_status(block_hash).await,
            ApiError::NoBlocksMatched,
        )
    }

    pub async fn query_block_info(
        &self,
        block_id: Option<Box<PartialBlockIdentifier>>,
    ) -> ApiResult<BlockInfo> {
        match block_id {
            None => {
                let consensus_status = self.client.clone().get_consensus_status().await?;
                let block_hash = consensus_status.last_finalized_block;
                self.query_block_info_by_hash(&block_hash).await
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
                            [block_hash] => self.query_block_info_by_hash(&block_hash).await,
                            _ => Err(ApiError::MultipleBlocksMatched),
                        }
                    }
                    (None, Some(hash)) => {
                        let block_hash = block_hash_from_string(hash.as_str())?;
                        self.query_block_info_by_hash(&block_hash).await
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

pub fn map_query_result<T>(res: Result<T, QueryError>, not_found_err: ApiError) -> ApiResult<T> {
    res.map_err(|err| map_query_error(err, not_found_err))
}

pub fn map_query_error(err: QueryError, not_found_err: ApiError) -> ApiError {
    match err {
        QueryError::RPCError(err) => err.into(),
        QueryError::NotFound => not_found_err,
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
    /// Real contract.
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
