use http_api_problem::*;
use rosetta::models::*;
use serde::Serialize;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::{reject, reply, Rejection, Reply};

use crate::api::network::Error;
use crate::NetworkApi;

pub async fn handle_rejection(rej: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = rej.find::<Error>() {
        match err {
            Error::UnsupportedNetworkIdentifier => Ok(HttpApiProblem::new(StatusCode::BAD_REQUEST)
                .title(err.to_string())
                .to_hyper_response()),
            Error::ClientRpcError(err) => {
                // TODO Assign status per error case.
                Ok(HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .title(err.to_string())
                    .to_hyper_response())
            }
        }
    } else {
        Err(rej)
    }
}

pub async fn network_list(api: NetworkApi, _: MetadataRequest) -> Result<impl Reply, Rejection> {
    to_json(api.network_list().await)
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

// TODO Can lift this function to remove the need for explicitly defining the above functions?
fn to_json(res: Result<impl Serialize, impl Reject>) -> Result<impl Reply, Rejection> {
    match res {
        Ok(val) => Ok(reply::json(&val)),
        Err(err) => Err(reject::custom(err)),
    }
}
