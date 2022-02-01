use anyhow::{Context, Result};
use clap::AppSettings;
use concordium_rust_sdk::endpoints::Client;
use rosetta::models::*;
use std::convert::Infallible;
use structopt::StructOpt;
use warp::Filter;

const ROSETTA_VERSION: &str = "1.4.10";
const NODE_VERSION: &str = "3.0.1";
const SERVER_VERSION: &str = "0.1.0";

#[derive(StructOpt)]
struct App {
    #[structopt(long = "port", help = "Listen port", default_value = "8080")]
    port: u16,
    #[structopt(
        long = "grpc-host",
        env = "GRPC_HOST",
        help = "Hostname or IP of the node's GRPC endpoint.",
        default_value = "localhost"
    )]
    grpc_host: String,
    #[structopt(
        long = "grpc-port",
        env = "GRPC_PORT",
        help = "Port of the node's GRPC endpoint.",
        default_value = "10000"
    )]
    grpc_port: u16,
    #[structopt(
        long = "grpc-token",
        env = "GRPC_TOKEN",
        help = "Access token of the node's GRPC endpoint.",
        default_value = "rpcadmin"
    )]
    grpc_token: String,
}

async fn list_networks(_: MetadataRequest) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&NetworkListResponse {
        network_identifiers: vec![NetworkIdentifier {
            blockchain: "concordium".to_string(),
            network: "mainnet".to_string(),
            sub_network_identifier: None,
        }],
    }))
}

async fn network_options(_: Client, _: NetworkRequest) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&NetworkOptionsResponse {
        version: Box::new(Version{
            rosetta_version: ROSETTA_VERSION.to_string(),
            node_version: NODE_VERSION.to_string(),
            middleware_version: Some(SERVER_VERSION.to_string()),
            metadata: None
        }),
        allow: Box::new(Default::default()),
    }))
}

// TODO How to change Client to mutable ref?
async fn network_status(client: Client, _: NetworkRequest) -> Result<impl warp::Reply, Infallible> {
    let result = client.clone().get_consensus_status().await.unwrap();
    Ok(warp::reply::json(&NetworkStatusResponse {
        current_block_identifier: Box::new(BlockIdentifier{
            index: result.last_finalized_block_height.height as i64,
            hash: result.last_finalized_block.to_string(),
        }),
        current_block_timestamp: result.last_finalized_time.unwrap().timestamp(),
        genesis_block_identifier: Box::new(BlockIdentifier{
            index: 0,
            hash: result.genesis_block.to_string()
        }),
        oldest_block_identifier: None,
        sync_status: None,
        peers: vec![],
    }))
}

fn with_client(client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = {
        let app = App::clap().global_setting(AppSettings::ColoredHelp);
        let matches = app.get_matches();
        App::from_clap(&matches)
    };

    let endpoint = tonic::transport::Endpoint::from_shared(format!(
        "http://{}:{}",
        app.grpc_host, app.grpc_port
    ))
    .context("invalid host and/or port")?;
    // Client is not mutated, but client methods take self as a mutable reference to ensure mutual exclusion.
    let client = Client::connect(endpoint, app.grpc_token)
        .await
        .context("cannot connect to node")?;

    println!("Listening on port {}.", app.port);

    let network_list_router = warp::path("list")
        .and(warp::body::json())
        .and_then(list_networks);
    let network_options_router = warp::path("options")
        .and(with_client(client.clone()))
        .and(warp::body::json())
        .and_then(network_options);
    let network_status_router = warp::path("status")
        .and(with_client(client.clone()))
        .and(warp::body::json())
        .and_then(network_status);
    let network_route = warp::path("network").and(
        network_list_router
            .or(network_options_router)
            .or(network_status_router),
    );
    let route = warp::post().and(network_route);

    warp::serve(route).run(([0, 0, 0, 0], app.port)).await;
    Ok(())
}
