use core::clone::Clone;
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use crate::handler;
use crate::api::network::NetworkApi;

fn network_list(
    api: NetworkApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("list")
        .and(with_network_api(api))
        .and(warp::body::json())
        .and_then(handler::network_list)
}

fn network_options(
    api: NetworkApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("options")
        .and(with_network_api(api))
        .and(warp::body::json())
        .and_then(handler::network_options)
}

fn network_status(
    api: NetworkApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("status")
        .and(with_network_api(api))
        .and(warp::body::json())
        .and_then(handler::network_status)
}

fn network(
    api: NetworkApi,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    warp::path("network").and(
        network_list(api.clone())
            .or(network_options(api.clone()))
            .or(network_status(api.clone())),
    )
}

pub fn root(
    api: NetworkApi,
) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    warp::post()
        .and(network(api))
        .recover(handler::handle_rejection)
}

fn with_network_api(
    api: NetworkApi,
) -> impl Filter<Extract = (NetworkApi,), Error = Infallible> + Clone {
    warp::any().map(move || api.clone())
}
