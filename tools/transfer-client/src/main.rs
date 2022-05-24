use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use clap::Parser;
use concordium_rust_sdk::types::transactions::ExactSizeTransactionSigner;
use reqwest::{blocking::*, Url};
use rosetta::models::*;
use serde_json::value::Value;
use std::{ops::Add, path::PathBuf};
use transfer_client::*;

#[derive(Parser, Debug)]
#[clap(
    author = "Concordium Foundation",
    about = "Client for sending a transfer transaction using the Rosetta implementation for the \
             Concordium blockchain.",
    version
)]
struct Args {
    #[clap(long = "url", help = "URL of Rosetta server.", default_value = "http://localhost:8080")]
    url:           String,
    #[clap(
        long = "network",
        help = "Network name. Used in network identifier.",
        default_value = "testnet"
    )]
    network:       String,
    #[clap(long = "sender", help = "Address of the account sending the transfer.")]
    sender_addr:   String,
    #[clap(long = "receiver", help = "Address of the account receiving the transfer.")]
    receiver_addr: String,
    #[clap(long = "amount", help = "Amount of μCCD to transfer.")]
    amount:        i64,
    #[clap(
        long = "keys-file",
        help = "Path of file containing the signing keys for the sender account."
    )]
    keys_file:     PathBuf,
    #[clap(long = "memo-hex", help = "Hex-encoded memo to attach to the transfer transaction.")]
    memo_hex:      Option<String>,
}

fn main() -> Result<()> {
    // Parse CLI args.
    let args = Args::parse();

    // Constants.
    let network_id = NetworkIdentifier {
        blockchain:             "concordium".to_string(),
        network:                args.network,
        sub_network_identifier: None,
    };

    // Configure HTTP client.
    let base_url = Url::parse(args.url.as_str())?;
    let client = Client::builder().connection_verbose(true).build()?;

    // Set up and load test data.
    let sender_keys = load_keys(&args.keys_file)?;
    let operations = test_transfer_operations(args.sender_addr, args.receiver_addr, args.amount);

    // Perform transfer.
    let preprocess_response =
        call_preprocess(client.clone(), &base_url, network_id.clone(), operations.clone())?;
    let metadata_response = call_metadata(
        client.clone(),
        &base_url,
        network_id.clone(),
        preprocess_response.options, // options from preprocess response must be passed directly
    )?;
    let metadata = serde_json::from_value::<Metadata>(metadata_response.metadata)?;
    let payload_metadata = serde_json::to_value(&Payload {
        account_nonce:      metadata.account_nonce,
        signature_count:    sender_keys.num_keys(),
        expiry_unix_millis: Utc::now().add(Duration::hours(2)).timestamp_millis() as u64,
        memo:               parse_memo(args.memo_hex)?,
    })?;
    let payloads_response = call_payloads(
        client.clone(),
        &base_url,
        network_id.clone(),
        operations.clone(),
        payload_metadata,
    )?;
    println!("unsigned transaction: {}", &payloads_response.unsigned_transaction);
    let parse_unsigned_response = call_parse(
        client.clone(),
        &base_url,
        network_id.clone(),
        false,
        payloads_response.unsigned_transaction.clone(),
    )?;
    println!(
        "parse unsigned response: {}",
        serde_json::to_string_pretty(&parse_unsigned_response)?
    );

    if parse_unsigned_response.operations != operations {
        return Err(anyhow!("failed comparison of unsigned parse"));
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
    println!(
        "parsed signed transaction: {}",
        serde_json::to_string_pretty(&parse_signed_response)?
    );

    if parse_signed_response.operations != operations {
        return Err(anyhow!("failed comparison of signed parse"));
    }

    let submit_response =
        call_submit(client, &base_url, network_id, combine_response.signed_transaction)?;
    println!("submit done: hash={}", submit_response.transaction_identifier.hash);
    Ok(())
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

fn test_transfer_operations(
    sender_addr: String,
    receiver_addr: String,
    amount: i64,
) -> Vec<Operation> {
    let currency = Box::new(Currency {
        symbol:   "CCD".to_string(),
        decimals: 6,
        metadata: None,
    });
    vec![
        Operation {
            operation_identifier: Box::new(OperationIdentifier {
                index:         0,
                network_index: None,
            }),
            related_operations:   None,
            _type:                "transfer".to_string(),
            status:               None,
            account:              Some(Box::new(AccountIdentifier {
                address:     sender_addr,
                sub_account: None,
                metadata:    None,
            })),
            amount:               Some(Box::new(Amount {
                value:    (-amount).to_string(),
                currency: currency.clone(),
                metadata: None,
            })),
            coin_change:          None,
            metadata:             None,
        },
        Operation {
            operation_identifier: Box::new(OperationIdentifier {
                index:         1,
                network_index: None,
            }),
            related_operations:   None,
            _type:                "transfer".to_string(),
            status:               None,
            account:              Some(Box::new(AccountIdentifier {
                address:     receiver_addr,
                sub_account: None,
                metadata:    None,
            })),
            amount:               Some(Box::new(Amount {
                value: amount.to_string(),
                currency,
                metadata: None,
            })),
            coin_change:          None,
            metadata:             None,
        },
    ]
}
