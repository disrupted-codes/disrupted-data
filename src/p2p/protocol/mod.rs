use libp2p::request_response::json;
use sha2::Digest;

use disrupted_data_sdk_rs::{ActionResult, Actions};

pub(crate) mod disrupted_data;
pub(crate) mod request_response;

mod kad;

pub(crate) enum EventHandlerOutcome {
    Ok,
    Failed(String),
}

pub(crate) type RequestResponseBehaviour = json::Behaviour<Actions, ActionResult>;

