use crate::{
    api::{amount::amount_from_uccd, error::ApiResult, query::QueryHelper},
    validate::network::NetworkValidator,
    AccountValidator,
};
use rosetta::models::*;
use std::ops::Deref;

#[derive(Clone)]
pub struct AccountApi {
    account_validator: AccountValidator,
    network_validator: NetworkValidator,
    query_helper:      QueryHelper,
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
        self.network_validator.validate_network_identifier(*req.network_identifier)?;
        self.account_validator.validate_currencies(req.currencies)?;
        let (block_info, amount) = self
            .query_helper
            .query_account_balance(req.block_identifier, req.account_identifier.deref())
            .await?;
        Ok(AccountBalanceResponse::new(
            BlockIdentifier::new(
                block_info.block_height.height as i64,
                block_info.block_hash.to_string(),
            ),
            vec![amount_from_uccd(amount.microccd as i128)],
        ))
    }
}
