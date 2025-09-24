use crate::{
    AccountApi,
    api::{block::BlockApi, construction::ConstructionApi},
};
use rosetta::models::*;
use serde::Serialize;
use std::convert::Infallible;
use warp::{Rejection, Reply, reject, reject::Reject, reply};

use crate::api::network::NetworkApi;

pub async fn network_list(api: NetworkApi, _: MetadataRequest) -> Result<impl Reply, Infallible> {
    Ok(reply::json(&api.network_list()))
}

pub async fn network_options(
    api: NetworkApi,
    req: NetworkRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.network_options(req).await)
}

pub async fn network_status(api: NetworkApi, req: NetworkRequest) -> Result<impl Reply, Rejection> {
    to_json(api.network_status(req).await)
}

pub async fn account_balance(
    api: AccountApi,
    req: AccountBalanceRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.account_balance(req).await)
}

pub async fn block(api: BlockApi, req: BlockRequest) -> Result<impl Reply, Rejection> {
    to_json(api.block(req).await)
}

pub async fn block_transaction(
    api: BlockApi,
    req: BlockTransactionRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.block_transaction(req).await)
}

pub async fn construction_preprocess(
    api: ConstructionApi,
    req: ConstructionPreprocessRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.preprocess(req).await)
}

pub async fn construction_metadata(
    api: ConstructionApi,
    req: ConstructionMetadataRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.metadata(req).await)
}

pub async fn construction_payloads(
    api: ConstructionApi,
    req: ConstructionPayloadsRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.payloads(req).await)
}

pub async fn construction_parse(
    api: ConstructionApi,
    req: ConstructionParseRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.parse(req).await)
}

pub async fn construction_combine(
    api: ConstructionApi,
    req: ConstructionCombineRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.combine(req).await)
}

pub async fn construction_submit(
    api: ConstructionApi,
    req: ConstructionSubmitRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.submit(req).await)
}

pub async fn construction_hash(
    api: ConstructionApi,
    req: ConstructionHashRequest,
) -> Result<impl Reply, Rejection> {
    to_json(api.hash(req).await)
}

// TODO Can lift this function to remove the need for explicitly defining the
// above functions?
fn to_json(res: Result<impl Serialize, impl Reject>) -> Result<impl Reply, Rejection> {
    match res {
        Ok(val) => Ok(reply::json(&val)),
        Err(err) => Err(reject::custom(err)),
    }
}
