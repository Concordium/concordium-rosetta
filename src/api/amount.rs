use crate::{
    api::error::{ApiError, ApiResult},
    validate::account::validate_currency,
};
use concordium_rust_sdk::protocol_level_tokens::AccountToken;
use rosetta::models::{Amount, Currency};
use std::ops::Deref;

pub fn amount_from_uccd(v: i128) -> Amount {
    Amount::new(v.to_string(), Currency::new("CCD".to_string(), 6))
}

pub fn amounts_from_plt_tokens(tokens: &Vec<AccountToken>) -> Vec<Amount> {
    tokens.iter().map(|plt_token| {
        Amount::new(plt_token.state.balance.value().to_string(), Currency::new(String::from(plt_token.token_id.clone()), i32::from(plt_token.state.balance.decimals())))
    }).collect()
}

pub fn uccd_from_amount(v: &Amount) -> ApiResult<i128> {
    validate_currency(v.currency.deref())?;
    v.value
        .parse()
        .map_err(|_| ApiError::InvalidAmount(v.value.clone()))
}
