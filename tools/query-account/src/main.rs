use anyhow::{Result, anyhow};
use clap::Parser;
use reqwest::{Url, blocking::*};
use rosetta::models::*;
use std::{thread::sleep, time::Duration};

#[derive(Parser, Debug)]
#[clap(
    author = "Concordium Foundation",
    about = "Client for querying the chain and filter out operations related to a given account.",
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
    #[clap(long = "address", help = "Address of the account.")]
    address: String,
}

fn main() -> Result<()> {
    // Parse CLI args.
    let args = Args::parse();
    let url = args.url;
    let network = args.network;
    let address = args.address;

    // Constants.
    let network_id = NetworkIdentifier::new("concordium".to_string(), network);

    // Configure HTTP client.
    let base_url = Url::parse(url.as_str())?;
    let client = Client::builder().connection_verbose(true).build()?;

    let mut next_from_height = 0;
    let mut computed_balance: i128 = call_rosetta_balance(
        client.clone(),
        &base_url,
        network_id.clone(),
        next_from_height,
        address.clone(),
    )
    .and_then(|v| v.balances[0].value.parse().map_err(anyhow::Error::new))
    .unwrap_or_default();
    loop {
        let status = call_rosetta_status(client.clone(), &base_url, network_id.clone())?;
        let current_block_height = status.current_block_identifier.index;
        if current_block_height <= next_from_height {
            eprintln!(
                "Reached the end of the chain at block height {}. Pausing for 10s...",
                current_block_height
            );
            sleep(Duration::from_secs(10));
            continue;
        }
        computed_balance = traverse_block_range(
            client.clone(),
            &base_url,
            network_id.clone(),
            next_from_height,
            current_block_height,
            address.clone(),
            computed_balance,
        )?;
        next_from_height = current_block_height + 1;
    }
}

fn traverse_block_range(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    from_block_height: i64,
    to_block_height: i64,
    address: String,
    mut computed_balance: i128,
) -> Result<i128> {
    eprintln!(
        "Querying blocks from height {} to {}...",
        from_block_height, to_block_height
    );
    for block_height in from_block_height..=to_block_height {
        if block_height % 100 == 0 {
            eprintln!("Querying block at height {}...", block_height);
        }
        let block_result =
            call_rosetta_block(client.clone(), base_url, network_id.clone(), block_height)?;
        let mut has_ops = false;
        if let Some(block) = block_result.block {
            for tx in block.transactions {
                for op in tx.operations {
                    if let Some(a) = op.account {
                        if a.address == address {
                            has_ops = true;
                            let amount = match op.amount.clone() {
                                None => 0,
                                Some(a) => a.value.parse()?,
                            };
                            computed_balance += amount;
                            println!(
                                "Account affected with operation type '{}' in transaction '{}' of \
                                 block '{}' at height {} with amount {}.",
                                op._type,
                                tx.transaction_identifier.hash,
                                block.block_identifier.hash,
                                block.block_identifier.index,
                                amount,
                            );
                        }
                    }
                }
            }
            if has_ops {
                println!(
                    "New computed balance: {}. Actual balance: {}.",
                    computed_balance,
                    call_rosetta_balance(
                        client.clone(),
                        base_url,
                        network_id.clone(),
                        block_height,
                        address.clone(),
                    )?
                    .balances[0]
                        .value
                );
            }
        }

        if let Some(ts) = block_result.other_transactions {
            if !ts.is_empty() {
                return Err(anyhow!("unexpected non-empty 'other_transaction'"));
            }
        }
    }
    Ok(computed_balance)
}

fn call_rosetta_status(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
) -> Result<NetworkStatusResponse> {
    let url = base_url.join("/network/status")?;
    client
        .post(url)
        .json(&NetworkRequest {
            network_identifier: Box::new(network_id),
            metadata: None,
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}

fn call_rosetta_block(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    block_height: i64,
) -> Result<BlockResponse> {
    let url = base_url.join("/block")?;
    client
        .post(url)
        .json(&BlockRequest {
            network_identifier: Box::new(network_id),
            block_identifier: Box::new(PartialBlockIdentifier {
                index: Some(block_height),
                hash: None,
            }),
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}

fn call_rosetta_balance(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    block_height: i64,
    address: String,
) -> Result<AccountBalanceResponse> {
    let url = base_url.join("/account/balance")?;
    client
        .post(url)
        .json(&AccountBalanceRequest {
            network_identifier: Box::new(network_id),
            block_identifier: Some(Box::new(PartialBlockIdentifier {
                index: Some(block_height),
                hash: None,
            })),
            account_identifier: Box::new(AccountIdentifier::new(address)),
            currencies: None,
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}
