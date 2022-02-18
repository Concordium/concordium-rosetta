use rosetta::models::{Amount, Currency};

pub fn amount_from_uccd(v: i64) -> Amount {
    Amount {
        value: v.to_string(),
        currency: Box::new(Currency {
            symbol: "CCD".to_string(),
            decimals: 6,
            metadata: None,
        }),
        metadata: None,
    }
}
