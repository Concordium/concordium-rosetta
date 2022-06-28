mod api;
mod handler;
mod handler_error;
mod route;
mod validate;
mod version;

use crate::{
    api::{
        account::AccountApi, block::BlockApi, construction::ConstructionApi, network::NetworkApi,
        query::QueryHelper,
    },
    validate::{account::AccountValidator, network::NetworkValidator},
};
use anyhow::{Context, Result};
use clap::Parser;
use concordium_rust_sdk::endpoints::Client;
use env_logger::{Builder, Env};
use rosetta::models::NetworkIdentifier;

#[derive(Parser, Debug)]
#[clap(
    author = "Concordium Foundation",
    about = "A server implementing the Rosetta API for the Concordium blockchain.",
    version
)]
struct Args {
    #[clap(
        long = "network",
        env = "CONCORDIUM_ROSETTA_NETWORK",
        help = "The name of the network that the connected node is part of; i.e. 'testnet' or \
                'mainnet'. Only requests with network identifier using this value will be \
                accepted (see docs for details)."
    )]
    network:    String,
    #[clap(
        long = "port",
        env = "CONCORDIUM_ROSETTA_PORT",
        help = "The port that HTTP requests are to be served on.",
        default_value = "8080"
    )]
    port:       u16,
    #[clap(
        long = "grpc-host",
        env = "CONCORDIUM_ROSETTA_GRPC_HOST",
        help = "Hostname or IP of the node's gRPC endpoint.",
        default_value = "localhost"
    )]
    grpc_host:  String,
    #[clap(
        long = "grpc-port",
        env = "CONCORDIUM_ROSETTA_GRPC_PORT",
        help = "Port of the node's gRPC endpoint.",
        default_value = "10000"
    )]
    grpc_port:  u16,
    #[clap(
        long = "grpc-token",
        env = "CONCORDIUM_ROSETTA_GRPC_TOKEN",
        help = "Access token of the node's gRPC endpoint.",
        default_value = "rpcadmin"
    )]
    grpc_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI args.
    let args = Args::parse();

    // Initialize logging.
    Builder::from_env(Env::default().default_filter_or("info")).init();

    // Initialize gRPC and client.
    let client =
        Client::connect(format!("http://{}:{}", args.grpc_host, args.grpc_port), args.grpc_token)
            .await
            .context("cannot connect to node")?;

    // Set up handlers.
    let network_validator = NetworkValidator::new(NetworkIdentifier {
        blockchain:             "concordium".to_string(),
        network:                args.network,
        sub_network_identifier: None,
    });
    let account_validator = AccountValidator {};
    let query_helper = QueryHelper::new(client);
    let network_api = NetworkApi::new(network_validator.clone(), query_helper.clone());
    let account_api =
        AccountApi::new(account_validator.clone(), network_validator.clone(), query_helper.clone());
    let block_api = BlockApi::new(network_validator.clone(), query_helper.clone());
    let construction_api = ConstructionApi::new(network_validator.clone(), query_helper.clone());

    // Configure and start web server.
    warp::serve(route::root(network_api, account_api, block_api, construction_api))
        .run(([0, 0, 0, 0], args.port))
        .await;
    Ok(())
}
