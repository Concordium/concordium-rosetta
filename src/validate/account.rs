use crate::api::error::{ApiError, ApiResult};
use rosetta::models::Currency;

#[derive(Clone)]
pub struct AccountValidator {}

impl AccountValidator {
    pub fn validate_currencies(&self, currencies: Option<Vec<Currency>>) -> ApiResult<()> {
        validate_currencies(currencies)
    }
}

pub fn validate_currencies(currencies: Option<Vec<Currency>>) -> ApiResult<()> {
    match currencies {
        None => Ok(()),
        Some(cs) => cs.iter().try_for_each(self::validate_currency),
    }
}

pub fn validate_currency(c: &Currency) -> ApiResult<()> {
    if !is_valid_currency(c) {
        return Err(ApiError::InvalidCurrency);
    }
    Ok(())
}

fn is_valid_currency(c: &Currency) -> bool {
    c.symbol == *"CCD" && c.decimals == 6
}
