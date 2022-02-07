use anyhow::Result;
use rosetta::models::*;
use serde::Serialize;
use warp::http::StatusCode;
use warp::{reply, Rejection, Reply};

use crate::NetworkService;

#[derive(Debug)]
struct TestCustomRejection;

impl warp::reject::Reject for TestCustomRejection {}

pub async fn handle_rejection(err: Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(_) = err.find::<TestCustomRejection>() {
        Ok(warp::reply::with_status(
            "test failure",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else {
        Err(err)
    }
}

pub async fn network_list(
    network_service: NetworkService,
    _: MetadataRequest,
) -> Result<impl Reply, Rejection> {
    match map_to_json(network_service.network_list().await) {
        Ok(val) => Ok(val),
        Err(_) => Err(warp::reject::custom(TestCustomRejection {})),
    }
}

pub async fn network_options(
    network_service: NetworkService,
    req: NetworkRequest,
) -> Result<impl Reply, Rejection> {
    match map_to_json(network_service.network_options(req).await) {
        Ok(val) => Ok(val),
        Err(_) => Err(warp::reject()),
    }
}

pub async fn network_status(
    network_service: NetworkService,
    req: NetworkRequest,
) -> Result<impl Reply, Rejection> {
    match map_to_json(network_service.network_status(req).await) {
        Ok(val) => Ok(val),
        Err(_) => Err(warp::reject()),
    }
}

// TODO Can map domain function directly to one composed with 'warp::reply::json'?
fn map_to_json<T: Serialize>(res: Result<T>) -> Result<impl Reply> {
    res.map(|r| reply::json(&r))
}
