use crate::api::amount::amount_from_uccd;
use crate::api::error::ApiResult;
use crate::api::query::QueryHelper;
use crate::validate::network::NetworkValidator;
use crate::AccountValidator;
use rosetta::models::*;
use std::ops::Deref;

#[derive(Clone)]
pub struct AccountApi {
    account_validator: AccountValidator,
    network_validator: NetworkValidator,
    query_helper: QueryHelper,
}

impl AccountApi {
    pub fn new(
        account_validator: AccountValidator,
        network_validator: NetworkValidator,
        query_helper: QueryHelper,
    ) -> Self {
        Self {
            account_validator,
            network_validator,
            query_helper,
        }
    }

    pub async fn account_balance(
        &self,
        req: AccountBalanceRequest,
    ) -> ApiResult<AccountBalanceResponse> {
        self.network_validator
            .validate_network_identifier(*req.network_identifier)?;
        self.account_validator.validate_currencies(req.currencies)?;
        let (block_info, account_info) = self
            .query_helper
            .query_account_info(req.block_identifier, req.account_identifier.deref())
            .await?;
        Ok(AccountBalanceResponse {
            block_identifier: Box::new(BlockIdentifier {
                index: block_info.block_height.height as i64,
                hash: block_info.block_hash.to_string(),
            }),
            balances: vec![amount_from_uccd(
                account_info.account_amount.microgtu as i64,
            )],
            metadata: None,
        })
    }
}
