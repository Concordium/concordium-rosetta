use anyhow::{Context, Result};
use clap::AppSettings;
use concordium_rust_sdk::endpoints::Client;
use rosetta::models::*;
use structopt::StructOpt;
use warp::http::StatusCode;
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

    let network_list_route = warp::path("list").map(|| {
        warp::reply::json(&NetworkListResponse {
            network_identifiers: vec![NetworkIdentifier {
                blockchain: "concordium".to_string(),
                network: "mainnet".to_string(),
                sub_network_identifier: None,
            }],
        })
    });
    let network_options_route =
        warp::path("options").map(|| warp::reply::with_status("<options result>", StatusCode::OK));
    let network_status_route =
        warp::path("status").map(|| warp::reply::with_status("<status result>", StatusCode::OK));
    let network_route = warp::path("network").and(
        network_list_route
            .or(network_options_route)
            .or(network_status_route),
    );

    println!("Listening on port {}.", app.port);
    warp::serve(warp::post().and(network_route))
        .run(([0, 0, 0, 0], app.port))
        .await;
    Ok(())
}
