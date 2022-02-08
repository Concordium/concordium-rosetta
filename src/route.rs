use core::clone::Clone;
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use crate::handler;
use crate::service::network::NetworkService;

fn network_list(
    network_service: NetworkService,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("list")
        .and(with_network_service(network_service))
        .and(warp::body::json())
        .and_then(handler::network_list)
}

fn network_options(
    network_service: NetworkService,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("options")
        .and(with_network_service(network_service))
        .and(warp::body::json())
        .and_then(handler::network_options)
}

fn network_status(
    network_service: NetworkService,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("status")
        .and(with_network_service(network_service))
        .and(warp::body::json())
        .and_then(handler::network_status)
}

fn network(
    network_service: NetworkService,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    warp::path("network").and(
        network_list(network_service.clone())
            .or(network_options(network_service.clone()))
            .or(network_status(network_service.clone())),
    )
}

pub fn root(
    network_service: NetworkService,
) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    warp::post()
        .and(network(network_service))
        .recover(handler::handle_rejection)
}

fn with_network_service(
    network_service: NetworkService,
) -> impl Filter<Extract = (NetworkService,), Error = Infallible> + Clone {
    warp::any().map(move || network_service.clone())
}
