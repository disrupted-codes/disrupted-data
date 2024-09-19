use std::iter;
use std::time::Duration;
use libp2p::{ping, request_response, StreamProtocol};
use libp2p::request_response::{json, ProtocolSupport};
use libp2p::swarm::NetworkBehaviour;

use crate::{ActionResult, Actions};

type RequestResponseEvent = request_response::Event<Actions, ActionResult>;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct UserNodeBehaviour {
    pub request_response: json::Behaviour<Actions, ActionResult>,
    pub ping: ping::Behaviour
}

impl UserNodeBehaviour {
    pub fn new() -> Self {
        let protocols = iter::once((StreamProtocol::new("/disrupted-data/browser/1"), ProtocolSupport::Full));
        let request_response_behaviour = json::Behaviour::<Actions, ActionResult>::new(protocols, request_response::Config::default().with_request_timeout(Duration::from_secs(30)));

        let ping_behaviour = ping::Behaviour::new(Default::default());

        Self {
            request_response: request_response_behaviour,
            ping: ping_behaviour
        }


    }
}

#[derive(Debug)]
pub enum Event {
    RequestResponse(RequestResponseEvent),
    Ping(ping::Event),

}


impl From<RequestResponseEvent> for Event {
    fn from(event: RequestResponseEvent) -> Self {
        Event::RequestResponse(event)
    }
}

impl From<ping::Event> for Event {
    fn from(event: ping::Event) -> Self {
        Event::Ping(event)
    }
}