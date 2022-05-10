use core::clone::Clone;
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use crate::{
    api::{block::BlockApi, network::NetworkApi},
    handler,
    handler_error::handle_rejection,
    AccountApi, ConstructionApi,
};

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

fn block_(api: BlockApi) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path::end().and(with_block_api(api)).and(warp::body::json()).and_then(handler::block)
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

fn construction_preprocess(
    api: ConstructionApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("preprocess")
        .and(warp::path::end())
        .and(with_construction_api(api))
        .and(warp::body::json())
        .and_then(handler::construction_preprocess)
}

fn construction_metadata(
    api: ConstructionApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("metadata")
        .and(warp::path::end())
        .and(with_construction_api(api))
        .and(warp::body::json())
        .and_then(handler::construction_metadata)
}

fn construction_payloads(
    api: ConstructionApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("payloads")
        .and(warp::path::end())
        .and(with_construction_api(api))
        .and(warp::body::json())
        .and_then(handler::construction_payloads)
}

fn construction_parse(
    api: ConstructionApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("parse")
        .and(warp::path::end())
        .and(with_construction_api(api))
        .and(warp::body::json())
        .and_then(handler::construction_parse)
}

fn construction_combine(
    api: ConstructionApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("combine")
        .and(warp::path::end())
        .and(with_construction_api(api))
        .and(warp::body::json())
        .and_then(handler::construction_combine)
}

fn construction_submit(
    api: ConstructionApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("submit")
        .and(warp::path::end())
        .and(with_construction_api(api))
        .and(warp::body::json())
        .and_then(handler::construction_submit)
}

fn construction_hash(
    api: ConstructionApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("hash")
        .and(warp::path::end())
        .and(with_construction_api(api))
        .and(warp::body::json())
        .and_then(handler::construction_hash)
}

fn network(api: NetworkApi) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("network")
        .and(network_list(api.clone()).or(network_options(api.clone())).or(network_status(api)))
}

fn account(api: AccountApi) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("account").and(account_balance(api))
}

fn block(api: BlockApi) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("block").and(block_(api.clone()).or(block_transaction(api)))
}

fn construction(
    api: ConstructionApi,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    warp::path("construction").and(
        construction_preprocess(api.clone())
            .or(construction_metadata(api.clone()))
            .or(construction_payloads(api.clone()))
            .or(construction_parse(api.clone()))
            .or(construction_combine(api.clone()))
            .or(construction_submit(api.clone()))
            .or(construction_hash(api)),
    )
}

pub fn root(
    network_api: NetworkApi,
    account_api: AccountApi,
    block_api: BlockApi,
    construction_api: ConstructionApi,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(
            network(network_api)
                .or(account(account_api))
                .or(block(block_api))
                .or(construction(construction_api)),
        )
        .with(warp::log("concordium_rosetta::route"))
        .recover(handle_rejection)
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

fn with_construction_api(
    api: ConstructionApi,
) -> impl Filter<Extract = (ConstructionApi,), Error = Infallible> + Clone {
    warp::any().map(move || api.clone())
}
