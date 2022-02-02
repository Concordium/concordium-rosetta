mod handler;
mod route;
mod version;

use anyhow::{Context, Result};
use clap::AppSettings;
use concordium_rust_sdk::endpoints::Client;
use structopt::StructOpt;

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

    let client = Client::connect(endpoint, app.grpc_token)
        .await
        .context("cannot connect to node")?;

    println!("Listening on port {}.", app.port);
    warp::serve(route::routes(client))
        .run(([0, 0, 0, 0], app.port))
        .await;
    Ok(())
}
