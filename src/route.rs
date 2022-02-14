use core::clone::Clone;
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use crate::api::network::NetworkApi;
use crate::{handler, AccountApi};

fn network_list(api: NetworkApi) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
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

fn network_status(api: NetworkApi) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("status")
        .and(with_network_api(api))
        .and(warp::body::json())
        .and_then(handler::network_status)
}

fn account_balance(
    api: AccountApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("balance")
        .and(with_account_api(api))
        .and(warp::body::json())
        .and_then(handler::account_balance)
}

fn account_coins() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("coins")
        .and(warp::body::json())
        .and_then(handler::account_coins)
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

fn account(
    api: AccountApi,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    warp::path("account").and(account_balance(api.clone()).or(account_coins()))
}

pub fn root(
    network_api: NetworkApi,
    account_api: AccountApi,
) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    warp::post()
        .and(network(network_api).or(account(account_api)))
        .recover(handler::handle_rejection)
}

fn with_network_api(
    api: NetworkApi,
) -> impl Filter<Extract = (NetworkApi,), Error = Infallible> + Clone {
    warp::any().map(move || api.clone())
}

fn with_account_api(
    api: AccountApi,
) -> impl Filter<Extract = (AccountApi,), Error = Infallible> + Clone {
    warp::any().map(move || api.clone())
}
