use crate::{
    api::{amount::{amount_from_uccd, amounts_from_plt_tokens}, error::ApiResult},
    validate::network::NetworkValidator,
    AccountValidator, QueryHelper,
};
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
        let (block_info, account_balance) = self
            .query_helper
            .query_account_balance(req.block_identifier, req.account_identifier.deref())
            .await?;

        // collect all balances of plt tokens into a vector, then also add the ccd balance
        let mut balances: Vec<Amount> = amounts_from_plt_tokens(&account_balance.tokens).into_iter().collect();
        balances.push(amount_from_uccd(account_balance.amount.micro_ccd as i128));

        Ok(AccountBalanceResponse::new(
            BlockIdentifier::new(
                block_info.block_height.height as i64,
                block_info.block_hash.to_string(),
            ),
            balances
        ))
    }
}
