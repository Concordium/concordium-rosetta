use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::Parser;
use concordium_rust_sdk::{
    common::types::{Amount, TransactionTime},
    id::types::AccountAddress,
    types::{
        transactions::{
            construct, construct::GivenEnergy, cost, BlockItem, ExactSizeTransactionSigner, Payload,
        },
        Memo, WalletAccount,
    },
    v2::{AccountIdentifier, Client, Endpoint},
};
use std::{convert::TryFrom, ops::Add, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(
    author = "Concordium Foundation",
    about = "Client for sending a transfer transaction using the Rosetta implementation for the \
             Concordium blockchain.",
    version
)]
struct Args {
    #[clap(
        long = "grpc-host",
        help = "Hostname or IP of the node's gRPC endpoint.",
        default_value = "localhost"
    )]
    grpc_host: String,
    #[clap(
        long = "grpc-port",
        help = "Port of the node's gRPC endpoint.",
        default_value = "10000"
    )]
    grpc_port: u16,
    #[clap(
        long = "sender-account-file",
        help = "Path of file containing the address and keys for the sender account."
    )]
    sender_account_file: PathBuf,
    #[clap(
        long = "receiver",
        help = "Address of the account receiving the transfer."
    )]
    receiver_address: AccountAddress,
    #[clap(long = "amount", help = "Amount of CCD to transfer.")]
    amount: Amount,
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

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI args.
    let args = Args::parse();
    let sender_account_file = args.sender_account_file;
    let grpc_host = args.grpc_host;
    let grpc_port = args.grpc_port;
    let to_address = args.receiver_address;
    let amount = args.amount;
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

    // Load sender keys.
    let sender_account = WalletAccount::from_json_file(sender_account_file)?;
    let from_address = sender_account.address;
    let sender_keys = sender_account.keys;

    // Configure client.
    let client = Client::new(Endpoint::from_shared(format!(
        "http://{}:{}",
        grpc_host, grpc_port
    ))?)
    .await
    .context("Cannot connect to the node.")?;

    // Configure and send transfer.
    let consensus_info = client
        .clone()
        .get_consensus_info()
        .await
        .context("cannot resolve latest block")?;
    let sender_info = client
        .clone()
        .get_account_info(
            &AccountIdentifier::Address(from_address),
            &consensus_info.last_finalized_block,
        )
        .await
        .context("cannot resolve next nonce of sender account")?;

    let payload = match memo {
        None => Payload::Transfer { to_address, amount },
        Some(m) => Payload::TransferWithMemo {
            to_address,
            amount,
            memo: m,
        },
    };
    let pre_tx = construct::make_transaction(
        from_address,
        sender_info.response.account_nonce,
        TransactionTime::from_seconds(Utc::now().add(Duration::hours(2)).timestamp_millis() as u64), /* TODO Make configurable. */
        GivenEnergy::Add {
            num_sigs: sender_keys.num_keys(),
            energy: cost::SIMPLE_TRANSFER,
        },
        payload,
    );
    let signed_tx = pre_tx.sign(&sender_keys);
    let block_item = BlockItem::AccountTransaction(signed_tx);
    let transaction_hash = client
        .clone()
        .send_block_item(&block_item)
        .await
        .context("cannot send transaction to node")?;

    println!("Sent transaction '{}'", transaction_hash);
    Ok(())
}
