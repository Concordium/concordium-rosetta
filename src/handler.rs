use anyhow::Result;
use concordium_rust_sdk::endpoints::Client;
use rosetta::models::*;
use serde::Serialize;
use std::convert::Infallible;
use std::error::Error;
use warp::{reply, Reply};

use crate::domain;

pub async fn network_list(_: MetadataRequest) -> Result<impl Reply, Infallible> {
    map_to_json(domain::network_list().await)
}

pub async fn network_options(
    client: Client,
    req: NetworkRequest,
) -> Result<impl Reply, Infallible> {
    map_to_json(domain::network_options(client, req).await)
}

pub async fn network_status(
    client: Client,
    req: NetworkRequest,
) -> Result<impl Reply, Infallible> {
    map_to_json(domain::network_status(client, req).await)
}

// TODO Can map domain function directly to one composed with 'warp::reply::json'?
fn map_to_json<T: Serialize, E: Error>(res: Result<T, E>) -> Result<impl Reply, E> {
    res.map(|r| reply::json(&r))
}
