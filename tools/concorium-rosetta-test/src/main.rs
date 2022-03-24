use anyhow::Result;
use chrono::{Duration, Utc};
use clap::AppSettings;
use concordium_rust_sdk::common::types::{CredentialIndex, KeyIndex};
use concordium_rust_sdk::id::types::AccountKeys;
use concordium_rust_sdk::types::transactions::ExactSizeTransactionSigner;
use concordium_rust_sdk::types::Memo;
use reqwest::blocking::*;
use reqwest::Url;
use rosetta::models::*;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use sign_transaction_hash::{load_keys, sign_hash};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::ops::Add;
use structopt::StructOpt;

#[derive(Deserialize)]
struct Metadata {
    account_nonce: u64,
}

#[derive(Serialize)]
struct Payload {
    account_nonce: u64,
    signature_count: u32,
    expiry_unix_millis: u64,
    memo: Option<Memo>,
}

#[derive(StructOpt)]
struct App {
    #[structopt(
        long = "url",
        help = "URL of Rosetta server.",
        default_value = "http://localhost:8080"
    )]
    url: String,
    #[structopt(
        long = "network",
        help = "Network name (supported values: 'mainnet').",
        default_value = "mainnet"
    )]
    network: String,
    #[structopt(long = "sender", help = "Address of the account sending a transfer.")]
    sender_addr: String,
    #[structopt(
        long = "receiver",
        help = "Address of the account receiving the transfer."
    )]
    receiver_addr: String,
    #[structopt(long = "amount", help = "Amount to transfer.")]
    amount: i64,
    #[structopt(
        long = "keys-file",
        help = "File containing the signing keys for the sender account."
    )]
    keys_file: String,
    #[structopt(
        long = "memo-hex",
        help = "Hex-encoded memo to attach to the transfer transaction."
    )]
    memo_hex: Option<String>,
}

type SignatureMap = BTreeMap<CredentialIndex, BTreeMap<KeyIndex, Signature>>;

fn main() -> Result<()> {
    // Parse CLI args.
    let app = {
        let app = App::clap().global_setting(AppSettings::ColoredHelp);
        let matches = app.get_matches();
        App::from_clap(&matches)
    };

    // Constants.
    let network_id = NetworkIdentifier {
        blockchain: "concordium".to_string(),
        network: app.network,
        sub_network_identifier: None,
    };

    // Configure HTTP client.
    let base_url = Url::parse(app.url.as_str())?;
    let client = Client::builder().connection_verbose(true).build()?;

    // Set up and load test data.
    let sender_keys = load_keys(&app.keys_file)?;
    let operations = test_transfer_operations(app.sender_addr, app.receiver_addr, app.amount);

    // Perform transfer.
    let preprocess_response = call_preprocess(
        client.clone(),
        &base_url,
        network_id.clone(),
        operations.clone(),
    )?;
    let metadata_response = call_metadata(
        client.clone(),
        &base_url,
        network_id.clone(),
        preprocess_response.options, // options from preprocess response must be passed directly
    )?;
    let metadata = serde_json::from_value::<Metadata>(metadata_response.metadata)?;
    let payload_metadata = serde_json::to_value(&Payload {
        account_nonce: metadata.account_nonce,
        signature_count: sender_keys.num_keys(),
        expiry_unix_millis: Utc::now().add(Duration::hours(2)).timestamp_millis() as u64,
        memo: parse_memo(app.memo_hex)?,
    })?;
    let payloads_response = call_payloads(
        client.clone(),
        &base_url,
        network_id.clone(),
        operations.clone(),
        payload_metadata,
    )?;
    println!(
        "unsigned transaction: {}",
        &payloads_response.unsigned_transaction,
    );
    let parse_unsigned_response = call_parse(
        client.clone(),
        &base_url,
        network_id.clone(),
        false,
        payloads_response.unsigned_transaction.clone(),
    )?;
    println!("parse unsigned response: {}", serde_json::to_string_pretty(&parse_unsigned_response)?);
    
    if parse_unsigned_response.operations != operations {
        return Err(anyhow::Error::msg("failed comparison of unsigned parse"));
    }

    let sigs =
        signature_maps_to_signatures(sign_payloads(payloads_response.payloads, &sender_keys)?);

    let combine_response = call_combine(
        client.clone(),
        &base_url,
        network_id.clone(),
        payloads_response.unsigned_transaction,
        sigs,
    )?;
    
    let parse_signed_response = call_parse(
        client.clone(),
        &base_url,
        network_id.clone(),
        true,
        combine_response.signed_transaction.clone(),
    )?;
    println!("parsed signed transaction: {}", serde_json::to_string_pretty(&parse_signed_response)?);
    
    if parse_signed_response.operations != operations {
        return Err(anyhow::Error::msg("failed comparison of signed parse"));
    }

    let submit_response = call_submit(
        client.clone(),
        &base_url,
        network_id.clone(),
        combine_response.signed_transaction,
    )?;
    println!(
        "submit done: hash={}",
        submit_response.transaction_identifier.hash
    );
    Ok(())
}

fn parse_memo(memo_hex: Option<String>) -> Result<Option<Memo>> {
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

fn signature_maps_to_signatures(signatures: Vec<SignatureMap>) -> Vec<Signature> {
    signatures
        .into_iter()
        .flat_map(|s| s.into_values().flat_map(|x| x.into_values()))
        .collect()
}

fn call_preprocess(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    operations: Vec<Operation>,
) -> Result<ConstructionPreprocessResponse> {
    // println!("calling preprocess");
    let url = base_url.join("/construction/preprocess")?;
    client
        .post(url)
        .json(&ConstructionPreprocessRequest {
            network_identifier: Box::new(network_id),
            operations,
            metadata: None,
            max_fee: None,
            suggested_fee_multiplier: None,
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}

fn call_metadata(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    options: Option<Value>,
) -> Result<ConstructionMetadataResponse> {
    // println!("calling metadata");
    let url = base_url.join("/construction/metadata")?;
    client
        .post(url)
        .json(&ConstructionMetadataRequest {
            network_identifier: Box::new(network_id),
            options,
            public_keys: None,
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}

fn call_payloads(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    operations: Vec<Operation>,
    metadata: Value,
) -> Result<ConstructionPayloadsResponse> {
    // println!("calling payloads");
    let url = base_url.join("/construction/payloads")?;
    client
        .post(url)
        .json(&ConstructionPayloadsRequest {
            network_identifier: Box::new(network_id),
            operations,
            metadata: Some(metadata),
            public_keys: None,
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}

fn call_parse(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    signed: bool,
    transaction: String,
) -> Result<ConstructionParseResponse> {
    // println!("calling parse");
    let url = base_url.join("/construction/parse")?;
    client
        .post(url)
        .json(&ConstructionParseRequest {
            network_identifier: Box::new(network_id),
            signed,
            transaction,
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}

fn call_combine(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    unsigned_transaction: String,
    signatures: Vec<Signature>,
) -> Result<ConstructionCombineResponse> {
    // println!("calling combine");
    let url = base_url.join("/construction/combine")?;
    client
        .post(url)
        .json(&ConstructionCombineRequest {
            network_identifier: Box::new(network_id),
            unsigned_transaction,
            signatures,
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}

fn call_submit(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    signed_transaction: String,
) -> Result<TransactionIdentifierResponse> {
    // println!("calling submit");
    let url = base_url.join("/construction/submit")?;
    client
        .post(url)
        .json(&ConstructionSubmitRequest {
            network_identifier: Box::new(network_id),
            signed_transaction,
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}

fn sign_payloads(payloads: Vec<SigningPayload>, keys: &AccountKeys) -> Result<Vec<SignatureMap>> {
    payloads.iter().map(|p| sign_payload(p, keys)).collect()
}

fn sign_payload(payload: &SigningPayload, keys: &AccountKeys) -> Result<SignatureMap> {
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
                            "{}/{}/{}",
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

fn test_transfer_operations(
    sender_addr: String,
    receiver_addr: String,
    amount: i64,
) -> Vec<Operation> {
    let currency = Box::new(Currency {
        symbol: "CCD".to_string(),
        decimals: 6,
        metadata: None,
    });
    vec![
        Operation {
            operation_identifier: Box::new(OperationIdentifier {
                index: 0,
                network_index: None,
            }),
            related_operations: None,
            _type: "transfer".to_string(),
            status: None,
            account: Some(Box::new(AccountIdentifier {
                address: sender_addr,
                sub_account: None,
                metadata: None,
            })),
            amount: Some(Box::new(Amount {
                value: (-amount).to_string(),
                currency: currency.clone(),
                metadata: None,
            })),
            coin_change: None,
            metadata: None,
        },
        Operation {
            operation_identifier: Box::new(OperationIdentifier {
                index: 1,
                network_index: None,
            }),
            related_operations: None,
            _type: "transfer".to_string(),
            status: None,
            account: Some(Box::new(AccountIdentifier {
                address: receiver_addr,
                sub_account: None,
                metadata: None,
            })),
            amount: Some(Box::new(Amount {
                value: amount.to_string(),
                currency: currency.clone(),
                metadata: None,
            })),
            coin_change: None,
            metadata: None,
        },
    ]
}
