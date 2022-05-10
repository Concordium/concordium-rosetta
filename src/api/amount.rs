use crate::{
    api::error::{ApiError, ApiResult},
    validate::account::validate_currency,
};
use rosetta::models::{Amount, Currency};
use std::ops::Deref;

pub fn amount_from_uccd(v: i128) -> Amount {
    Amount::new(v.to_string(), Currency::new("CCD".to_string(), 6))
}

pub fn uccd_from_amount(v: &Amount) -> ApiResult<i128> {
    validate_currency(v.currency.deref())?;
    v.value.parse().map_err(|_| ApiError::InvalidAmount(v.value.clone()))
}
