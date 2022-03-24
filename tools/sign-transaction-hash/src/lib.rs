use anyhow::{Context, Result};
use concordium_rust_sdk::common::types::TransactionSignature;
use concordium_rust_sdk::id::types::AccountKeys;
use concordium_rust_sdk::types::hashes::TransactionSignHash;
use concordium_rust_sdk::types::transactions::TransactionSigner;
use std::fs;
use std::str::FromStr;

pub fn load_keys(path: &String) -> Result<AccountKeys> {
    let data = fs::read_to_string(path).context("cannot read file")?;
    serde_json::from_str(&data).context("cannot parse keys read from file")
}

pub fn sign_hash(keys: &AccountKeys, hash: &str) -> Result<TransactionSignature> {
    let tx_hash = TransactionSignHash::from_str(hash).context("cannot parse hash from input")?;
    Ok(keys.sign_transaction_hash(&tx_hash))
}
