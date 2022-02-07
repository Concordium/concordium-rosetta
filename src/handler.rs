use anyhow::Result;
use rosetta::models::*;
use serde::Serialize;
use std::convert::Infallible;
use std::error::Error;
use warp::{reply, Reply};

use crate::NetworkService;

pub async fn network_list(
    network_service: NetworkService,
    _: MetadataRequest,
) -> Result<impl Reply, Infallible> {
    map_to_json(network_service.network_list().await)
}

pub async fn network_options(
    network_service: NetworkService,
    req: NetworkRequest,
) -> Result<impl Reply, Infallible> {
    map_to_json(network_service.network_options(req).await)
}

pub async fn network_status(
    network_service: NetworkService,
    req: NetworkRequest,
) -> Result<impl Reply, Infallible> {
    map_to_json(network_service.network_status(req).await)
}

// TODO Can map domain function directly to one composed with 'warp::reply::json'?
fn map_to_json<T: Serialize, E: Error>(res: Result<T, E>) -> Result<impl Reply, E> {
    res.map(|r| reply::json(&r))
}
