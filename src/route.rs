use core::clone::Clone;
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use crate::api::block::BlockApi;
use crate::api::network::NetworkApi;
use crate::{handler, AccountApi};

fn network_list(
    api: NetworkApi,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("list")
        .and(warp::path::end())
        .and(with_network_api(api))
        .and(warp::body::json())
        .and_then(handler::network_list)
}

fn network_options(
    api: NetworkApi,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("options")
        .and(warp::path::end())
        .and(with_network_api(api))
        .and(warp::body::json())
        .and_then(handler::network_options)
}

fn network_status(
    api: NetworkApi,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("status")
        .and(warp::path::end())
        .and(with_network_api(api))
        .and(warp::body::json())
        .and_then(handler::network_status)
}

fn account_balance(
    api: AccountApi,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("balance")
        .and(warp::path::end())
        .and(with_account_api(api))
        .and(warp::body::json())
        .and_then(handler::account_balance)
}

fn account_coins() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("coins")
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(handler::account_coins)
}

fn block_(api: BlockApi) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path::end()
        .and(with_block_api(api))
        .and(warp::body::json())
        .and_then(handler::block)
}

fn block_transaction(
    api: BlockApi,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("transaction")
        .and(warp::path::end())
        .and(with_block_api(api))
        .and(warp::body::json())
        .and_then(handler::block_transaction)
}

fn network(api: NetworkApi) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("network").and(
        network_list(api.clone())
            .or(network_options(api.clone()))
            .or(network_status(api.clone())),
    )
}

fn account(api: AccountApi) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("account").and(account_balance(api.clone()).or(account_coins()))
}

fn block(api: BlockApi) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("block").and(block_(api.clone()).or(block_transaction(api.clone())))
}

pub fn root(
    network_api: NetworkApi,
    account_api: AccountApi,
    block_api: BlockApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(
            network(network_api)
                .or(account(account_api))
                .or(block(block_api)),
        )
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

fn with_block_api(api: BlockApi) -> impl Filter<Extract = (BlockApi,), Error = Infallible> + Clone {
    warp::any().map(move || api.clone())
}
