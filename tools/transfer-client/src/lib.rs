use anyhow::{Context, Result};
use concordium_rust_sdk::common::types::{CredentialIndex, KeyIndex, TransactionSignature};
use concordium_rust_sdk::id::types::AccountKeys;
use concordium_rust_sdk::types::Memo;
use concordium_rust_sdk::types::hashes::TransactionSignHash;
use concordium_rust_sdk::types::transactions::TransactionSigner;
use rosetta::models::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fs;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct Metadata {
    pub account_nonce: u64,
}

#[derive(Serialize)]
pub struct Payload {
    pub account_nonce: u64,
    pub signature_count: u32,
    pub expiry_unix_millis: u64,
    pub memo: Option<Memo>,
}

pub fn parse_memo(memo_hex: Option<String>) -> Result<Option<Memo>> {
    let memo = match memo_hex {
        None => None,
        Some(s) => {
            let bs = hex::decode(s)?;
            let m = Memo::try_from(bs)?;
            Some(m)
        }
    };
    Ok(memo)
}

type SignatureMap = BTreeMap<CredentialIndex, BTreeMap<KeyIndex, Signature>>;

pub fn sign_payloads(payloads: Vec<SigningPayload>, keys: &AccountKeys) -> Result<Vec<SignatureMap>> {
    payloads.iter().map(|p| sign_payload(p, keys)).collect()
}

pub fn sign_payload(payload: &SigningPayload, keys: &AccountKeys) -> Result<SignatureMap> {
    let res = sign_hash(keys, payload.hex_bytes.as_str())?
        .signatures
        .iter()
        .map(|(cred_idx, sigs)| {
            (
                *cred_idx,
                sigs.iter()
                    .map(|(key_idx, sig)| {
                        let public_key = keys.keys[cred_idx].keys[key_idx].public;
                        let public_key_hex = hex::encode(public_key);
                        let sig_hex = format!(
                            "{}:{}/{}",
                            cred_idx.index,
                            u8::from(*key_idx),
                            hex::encode(sig)
                        );
                        (
                            *key_idx,
                            Signature {
                                signing_payload: Box::new(payload.clone()),
                                public_key: Box::new(PublicKey {
                                    hex_bytes: public_key_hex,
                                    curve_type: CurveType::Edwards25519,
                                }),
                                signature_type: SignatureType::Ed25519,
                                hex_bytes: sig_hex,
                            },
                        )
                    })
                    .collect(),
            )
        })
        .collect();
    Ok(res)
}

pub fn sign_hash(keys: &AccountKeys, hash: &str) -> Result<TransactionSignature> {
    let tx_hash = TransactionSignHash::from_str(hash).context("cannot parse hash from input")?;
    Ok(keys.sign_transaction_hash(&tx_hash))
}

pub fn load_keys(path: &String) -> Result<AccountKeys> {
    let data = fs::read_to_string(path).context("cannot read file")?;
    serde_json::from_str(&data).context("cannot parse keys read from file")
}

pub fn signature_maps_to_signatures(signatures: Vec<SignatureMap>) -> Vec<Signature> {
    signatures
        .into_iter()
        .flat_map(|s| s.values().flat_map(|x| x.values().cloned()).collect::<Vec<Signature>>())
        .collect()
}
