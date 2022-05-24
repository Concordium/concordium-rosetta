use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use clap::Parser;
use concordium_rust_sdk::{
    common::types::{Amount, TransactionTime},
    constants::DEFAULT_NETWORK_ID,
    endpoints::Client,
    id::types::{AccountAddress, AccountKeys},
    types::{
        transactions::{
            construct, construct::GivenEnergy, cost, BlockItem, ExactSizeTransactionSigner, Payload,
        },
        Memo,
    },
};
use std::{convert::TryFrom, fs, ops::Add, path::PathBuf};
use tonic::transport::Endpoint;

#[derive(Parser, Debug)]
#[clap(
    author = "Concordium Foundation",
    about = "Client for sending a transfer transaction using the Rosetta implementation for the \
             Concordium blockchain.",
    version
)]
struct Args {
    #[clap(
        long = "node",
        help = "Endpoint (<hostname or IP>:<port>) of the node's gRPC endpoint.",
        default_value = "localhost:8080"
    )]
    node:             Endpoint,
    #[clap(
        long = "grpc-token",
        help = "Access token of the node's gRPC endpoint.",
        default_value = "rpcadmin"
    )]
    grpc_token:       String,
    #[clap(long = "sender", help = "Address of the account sending the transfer.")]
    sender_address:   AccountAddress,
    #[clap(long = "receiver", help = "Address of the account receiving the transfer.")]
    receiver_address: AccountAddress,
    #[clap(long = "amount", help = "Amount of CCD to transfer.")]
    amount:           Amount,
    #[clap(
        long = "keys-file",
        help = "Path of file containing the signing keys for the sender account."
    )]
    sender_keys_file: PathBuf,
    #[clap(long = "memo-hex", help = "Hex-encoded memo to attach to the transfer transaction.")]
    memo_hex:         Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI args.
    let args = Args::parse();
    let sender_keys_file = args.sender_keys_file;
    let grpc_endpoint = args.node;
    let grpc_token = args.grpc_token;
    let to_address = args.receiver_address;
    let from_address = args.sender_address;
    let amount = args.amount;
    let memo = args.memo_hex;

    // Load sender keys.
    let sender_keys_json =
        fs::read_to_string(&sender_keys_file).context("cannot read keys file")?;
    let sender_keys: AccountKeys =
        serde_json::from_str(&sender_keys_json).context("cannot parse keys loaded from file")?;

    // Configure client.
    let client =
        Client::connect(grpc_endpoint, grpc_token).await.context("cannot connect to node")?;

    // Configure and send transfer.
    let consensus_status =
        client.clone().get_consensus_status().await.context("cannot resolve latest block")?;
    let sender_info = client
        .clone()
        .get_account_info(from_address, &consensus_status.last_finalized_block)
        .await
        .context("cannot resolve next nonce of sender account")?;

    let payload = match memo {
        None => Payload::Transfer {
            to_address,
            amount,
        },
        Some(memo) => Payload::TransferWithMemo {
            to_address,
            amount,
            memo: Memo::try_from(hex::decode(memo)?)?,
        },
    };
    let pre_tx = construct::make_transaction(
        from_address,
        sender_info.account_nonce,
        TransactionTime::from_seconds(Utc::now().add(Duration::hours(2)).timestamp_millis() as u64), /* TODO Make configurable. */
        GivenEnergy::Add {
            num_sigs: sender_keys.num_keys(),
            energy:   cost::SIMPLE_TRANSFER,
        },
        payload,
    );
    let signed_tx = pre_tx.sign(&sender_keys);
    let block_item = BlockItem::AccountTransaction(signed_tx);
    let success = client
        .clone()
        .send_transaction(DEFAULT_NETWORK_ID, &block_item)
        .await
        .context("cannot send transaction to node")?;
    if !success {
        return Err(anyhow!("node rejected transaction"));
    }

    println!("Sent transaction '{}'", block_item.hash());
    Ok(())
}
