use crate::api::error::{ApiError, ApiResult};
use rosetta::models::Currency;

#[derive(Clone)]
pub struct AccountValidator {}

impl AccountValidator {
    pub fn new() {}

    pub fn validate_currencies(&self, currencies: Option<Vec<Currency>>) -> ApiResult<()> {
        match currencies {
            None => Ok(()),
            Some(cs) => match cs.iter().find(|c| c.symbol != *"CCD" || c.decimals != 6) {
                None => Ok(()),
                Some(_) => Err(ApiError::InvalidCurrency),
            },
        }
    }
}
