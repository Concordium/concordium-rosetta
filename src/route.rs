use concordium_rust_sdk::endpoints::Client;
use core::clone::Clone;
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use crate::handler;

fn network_list() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("list")
        .and(warp::body::json())
        .and_then(handler::network_list)
}

fn network_options(client: Client) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("options")
        .and(with_client(client))
        .and(warp::body::json())
        .and_then(handler::network_options)
}

fn network_status(client: Client) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("status")
        .and(with_client(client))
        .and(warp::body::json())
        .and_then(handler::network_status)
}

fn network(
    client: Client,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    warp::path("network").and(
        network_list()
            .or(network_options(client.clone()))
            .or(network_status(client.clone())),
    )
}

pub fn root(client: Client) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    warp::post().and(network(client.clone()))
}

fn with_client(client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}
