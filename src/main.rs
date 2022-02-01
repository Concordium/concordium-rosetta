use anyhow::{Context, Result};
use clap::AppSettings;
use concordium_rust_sdk::endpoints::Client;
use rosetta::models::*;
use std::convert::Infallible;
use structopt::StructOpt;
use warp::Filter;

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

async fn network_options(_: NetworkRequest) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&NetworkOptionsResponse {
        version: Box::new(Default::default()),
        allow: Box::new(Default::default()),
    }))
}

async fn network_status(_: NetworkRequest) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&NetworkStatusResponse {
        current_block_identifier: Box::new(Default::default()),
        current_block_timestamp: 0,
        genesis_block_identifier: Box::new(Default::default()),
        oldest_block_identifier: None,
        sync_status: None,
        peers: vec![],
    }))
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
    let _client = Client::connect(endpoint, app.grpc_token)
        .await
        .context("cannot connect to node")?;

    println!("Listening on port {}.", app.port);

    let network_list_router = warp::path("list")
        .and(warp::body::json())
        .and_then(list_networks);
    let network_options_router = warp::path("options")
        .and(warp::body::json())
        .and_then(network_options);
    let network_status_router = warp::path("status")
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
