use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use clap::Parser;
use concordium_rust_sdk::{
    common::types::Amount as ConcordiumAmount,
    types::{transactions::ExactSizeTransactionSigner, Memo, WalletAccount},
};
use reqwest::{blocking::*, Url};
use rosetta::models::*;
use serde_json::value::Value;
use std::{convert::TryFrom, ops::Add, path::PathBuf};
use transfer_client::*;

#[derive(Parser, Debug)]
#[clap(
    author = "Concordium Foundation",
    about = "Client for sending a transfer transaction using the Rosetta implementation for the \
             Concordium blockchain.",
    version
)]
struct Args {
    #[clap(
        long = "url",
        help = "URL of Rosetta server.",
        default_value = "http://localhost:8080"
    )]
    url: String,
    #[clap(
        long = "network",
        help = "Network name. Used in network identifier.",
        default_value = "testnet"
    )]
    network: String,
    #[clap(
        long = "sender-account-file",
        help = "Path of file containing the address and keys for the sender account."
    )]
    sender_account_file: PathBuf,
    #[clap(
        long = "receiver",
        help = "Address of the account receiving the transfer."
    )]
    receiver_addr: String,
    #[clap(long = "amount", help = "Amount of CCD to transfer.")]
    amount: ConcordiumAmount,
    #[clap(
        long = "memo-hex",
        help = "Hex-encoded memo to attach to the transaction.",
        group = "memo"
    )]
    memo_hex: Option<String>,
    #[clap(
        long = "memo-string",
        help = "Memo string to attach (CBOR-encoded) to the transaction.",
        group = "memo"
    )]
    memo_str: Option<String>,
}

fn main() -> Result<()> {
    // Parse CLI args.
    let args = Args::parse();

    let url = args.url;
    let network = args.network;
    let sender_account_file = args.sender_account_file;

    let memo_bytes = match (args.memo_hex, args.memo_str) {
        (None, None) => None,
        (Some(hex), None) => Some(hex::decode(hex)?),
        (None, Some(str)) => {
            let mut buf = Vec::new();
            serde_cbor::to_writer(&mut buf, &serde_cbor::Value::Text(str))?;
            Some(buf)
        }
        (Some(_), Some(_)) => unreachable!(),
    };
    let memo = memo_bytes.map(Memo::try_from).transpose()?;

    // Constants.
    let network_id = NetworkIdentifier::new("concordium".to_string(), network);

    // Configure HTTP client.
    let base_url = Url::parse(url.as_str())?;
    let client = Client::builder().connection_verbose(true).build()?;

    // Set up and load test data.
    let sender_account = WalletAccount::from_json_file(sender_account_file)?;
    let receiver_addr = args.receiver_addr;
    let amount = args.amount;
    let operations = test_transfer_operations(
        sender_account.address.to_string(),
        receiver_addr,
        amount.micro_ccd as i64,
    );

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
    let payload_metadata = serde_json::to_value(Payload {
        account_nonce: metadata.account_nonce,
        signature_count: sender_account.num_keys(),
        expiry_unix_millis: Utc::now().add(Duration::hours(2)).timestamp_millis() as u64,
        memo,
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
        &payloads_response.unsigned_transaction
    );
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

    let sigs = signature_maps_to_signatures(sign_payloads(
        payloads_response.payloads,
        &sender_account.keys,
    )?);

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

    let submit_response = call_submit(
        client,
        &base_url,
        network_id,
        combine_response.signed_transaction,
    )?;
    println!(
        "submit done: hash={}",
        submit_response.transaction_identifier.hash
    );
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
                currency,
                metadata: None,
            })),
            coin_change: None,
            metadata: None,
        },
    ]
}
