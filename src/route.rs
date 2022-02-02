use concordium_rust_sdk::endpoints::Client;
use core::clone::Clone;
use std::convert::Infallible;
use warp::{Filter, Rejection};

use crate::handler;

fn with_client(client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn network_routes(
    client: Client,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    let network_list_router = warp::path("list")
        .and(warp::body::json())
        .and_then(handler::list_networks);
    let network_options_router = warp::path("options")
        .and(with_client(client.clone()))
        .and(warp::body::json())
        .and_then(handler::network_options);
    let network_status_router = warp::path("status")
        .and(with_client(client.clone()))
        .and(warp::body::json())
        .and_then(handler::network_status);
    warp::path("network").and(
        network_list_router
            .or(network_options_router)
            .or(network_status_router),
    )
}

pub fn routes(
    client: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post().and(network_routes(client.clone()))
}
