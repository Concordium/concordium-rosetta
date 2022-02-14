use crate::api::error::{ApiError, ApiResult, InvalidPartialBlockIdentifier};
use crate::validate::network::NetworkValidator;
use crate::AccountValidator;
use concordium_rust_sdk::endpoints::{BlocksAtHeightInput, Client};
use concordium_rust_sdk::id::types::AccountAddress;
use concordium_rust_sdk::types::hashes::BlockHash;
use concordium_rust_sdk::types::queries::BlockInfo;
use concordium_rust_sdk::types::AbsoluteBlockHeight;
use rosetta::models::*;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Clone)]
pub struct AccountApi {
    account_validator: AccountValidator,
    network_validator: NetworkValidator,
    client: Client,
}

impl AccountApi {
    pub fn new(
        account_validator: AccountValidator,
        network_validator: NetworkValidator,
        client: Client,
    ) -> Self {
        Self {
            account_validator,
            network_validator,
            client,
        }
    }

    pub async fn account_balance(
        &self,
        req: AccountBalanceRequest,
    ) -> ApiResult<AccountBalanceResponse> {
        self.network_validator
            .validate_network_identifier(*req.network_identifier)?;
        self.account_validator.validate_currencies(req.currencies)?;
        let block = self.resolve_block(req.block_identifier).await?;
        let block_hash = block_hash_from_string(block.block_hash.to_string().as_str())?;
        let address = account_address_from_identifier(req.account_identifier.deref())?;
        let account_info = self
            .client
            .clone()
            .get_account_info(address, &block_hash)
            .await?;
        Ok(AccountBalanceResponse {
            block_identifier: Box::new(BlockIdentifier {
                index: block.block_height.height as i64,
                hash: block.block_hash.to_string(),
            }),
            balances: vec![Amount {
                value: account_info.account_amount.microgtu.to_string(),
                currency: Box::new(Currency {
                    symbol: "CCD".to_string(),
                    decimals: 6,
                    metadata: None,
                }),
                metadata: None,
            }],
            metadata: None,
        })
    }

    async fn resolve_block(
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
                            return Err(ApiError::InvalidPartialBlockIdentifier(
                                InvalidPartialBlockIdentifier::InvalidIndex,
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
                    (Some(_), Some(_)) => Err(ApiError::InvalidPartialBlockIdentifier(
                        InvalidPartialBlockIdentifier::InconsistentValues,
                    )),
                    (None, None) => Err(ApiError::InvalidPartialBlockIdentifier(
                        InvalidPartialBlockIdentifier::NoValues,
                    )),
                }
            }
        }
    }
}

fn block_hash_from_string(hash: &str) -> ApiResult<BlockHash> {
    BlockHash::from_str(hash).map_err(|_| {
        ApiError::InvalidPartialBlockIdentifier(InvalidPartialBlockIdentifier::InvalidHash)
    })
}

fn account_address_from_identifier(id: &AccountIdentifier) -> ApiResult<AccountAddress> {
    match id.sub_account {
        None => account_address_from_string(id.address.as_str()),
        Some(_) => Err(ApiError::SubAccountNotImplemented),
    }
}

fn account_address_from_string(addr: &str) -> ApiResult<AccountAddress> {
    AccountAddress::from_str(addr).map_err(|_| ApiError::InvalidAccountAddress)
}
