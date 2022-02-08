use http_api_problem::*;
use rosetta::models::*;
use serde::Serialize;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::{reject, reply, Rejection, Reply};

use crate::service::network::ServiceError::{self, UnsupportedNetworkIdentifier};
use crate::NetworkService;

pub async fn handle_rejection(rej: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = rej.find::<ServiceError>() {
        match err {
            UnsupportedNetworkIdentifier => Ok(HttpApiProblem::new(StatusCode::BAD_REQUEST)
                .title(err.to_string())
                .to_hyper_response()),
        }
    } else {
        Err(rej)
    }
}

pub async fn network_list(
    network_service: NetworkService,
    _: MetadataRequest,
) -> Result<impl Reply, Rejection> {
    to_json(network_service.network_list().await)
}

pub async fn network_options(
    network_service: NetworkService,
    req: NetworkRequest,
) -> Result<impl Reply, Rejection> {
    to_json(network_service.network_options(req).await)
}

pub async fn network_status(
    network_service: NetworkService,
    req: NetworkRequest,
) -> Result<impl Reply, Rejection> {
    to_json(network_service.network_status(req).await)
}

// TODO Can lift this function to remove the need for explicitly define above functions?
fn to_json<T: Serialize, E: Reject>(res: Result<T, E>) -> Result<impl Reply, Rejection> {
    match res {
        Ok(val) => Ok(reply::json(&val)),
        Err(err) => Err(reject::custom(err)),
    }
}
