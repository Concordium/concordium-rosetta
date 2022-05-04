use anyhow::Result;
use clap::Parser;
use reqwest::{blocking::*, Url};
use rosetta::models::*;

#[derive(Parser, Debug)]
#[clap(
    author = "Concordium Foundation",
    about = "Client for querying the chain and filter out operations related to a given account.",
    version
)]
struct Args {
    #[clap(long = "url", help = "URL of Rosetta server.", default_value = "http://localhost:8080")]
    url:     String,
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

    // Constants.
    let network_id = NetworkIdentifier {
        blockchain:             "concordium".to_string(),
        network:                args.network,
        sub_network_identifier: None,
    };

    // Configure HTTP client.
    let base_url = Url::parse(args.url.as_str())?;
    let client = Client::builder().connection_verbose(true).build()?;

    // TODO Call consensus status to get last block height (iteratively).
    call_blocks(client, &base_url, network_id, 0, 1_000_000, args.address)?;

    Ok(())
}

fn call_blocks(
    client: Client,
    base_url: &Url,
    network_id: NetworkIdentifier,
    from_block_height: i64,
    to_block_height: i64,
    address: String,
) -> Result<()> {
    for block_height in from_block_height..=to_block_height {
        if block_height % 100 == 0 {
            println!("Querying block at height {}.", block_height);
        }
        let block_result = call_block(client.clone(), base_url, network_id.clone(), block_height)?;
        if let Some(block) = block_result.block {
            for tx in block.transactions {
                for op in tx.operations {
                    if let Some(a) = op.account {
                        if a.address == address {
                            println!(
                                "Account affected with operation type '{}' in transaction '{}' of \
                                 block '{}' at height {}.",
                                op._type,
                                tx.transaction_identifier.hash,
                                block.block_identifier.hash,
                                block.block_identifier.index
                            );
                        }
                    }
                }
            }
        }

        if let Some(ts) = block_result.other_transactions {
            if !ts.is_empty() {
                println!("unexpected non-empty 'other_transaction'...");
            }
        }
    }
    Ok(())
}

fn call_block(
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
            block_identifier:   Box::new(PartialBlockIdentifier {
                index: Some(block_height),
                hash:  None,
            }),
        })
        .send()?
        .json()
        .map_err(reqwest::Error::into)
}
