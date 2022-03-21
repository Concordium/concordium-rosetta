use rosetta::models::{Amount, Currency};

pub fn amount_from_uccd(v: i64) -> Amount {
    Amount::new(v.to_string(), Currency::new("CCD".to_string(), 6))
}
