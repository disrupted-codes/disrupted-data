use std::iter;
use std::time::Duration;

use libp2p::{identify, kad, PeerId, ping, request_response, StreamProtocol};
use libp2p::identity::Keypair;
use libp2p::kad::Mode::Server;
use libp2p::kad::store::MemoryStore;
use libp2p::request_response::{json, ProtocolSupport};
use libp2p::swarm::NetworkBehaviour;

use disrupted_data_sdk_rs::{ActionResult, Actions};

use crate::p2p::protocol::RequestResponseBehaviour;

type RequestResponseEvent = request_response::Event<Actions, ActionResult>;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct Behaviour {
    pub request_response: json::Behaviour<Actions, ActionResult>,
    pub kad: kad::Behaviour<MemoryStore>,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
}

impl Behaviour {
    pub fn new(peer_id: &PeerId, key_pair: &Keypair) -> Self {
        let store = MemoryStore::new(*peer_id);
        let mut kademlia = kad::Behaviour::new(*peer_id, store);
        kademlia.set_mode(Option::from(Server));

        let protocol = StreamProtocol::new("/client/1");
        let protocols = iter::once((protocol, ProtocolSupport::Full));
        let request_response_behaviour = RequestResponseBehaviour::new(protocols, request_response::Config::default());


        let ping = ping::Behaviour::new(Default::default());

        let identify_config = identify::Config::new(
            "/ipfs/id/1.0.0".to_string(),
            key_pair.public(),
        ).with_interval(Duration::from_secs(20));

        let identify = identify::Behaviour::new(identify_config);

        Behaviour {
            kad: kademlia,
            identify,
            request_response: request_response_behaviour,
            ping,
        }
    }

}
#[derive(Debug)]
pub enum Event {
    Kademlia(kad::Event),
    Identify(identify::Event),
    RequestResponse(RequestResponseEvent),
    Ping(ping::Event),

}


impl From<kad::Event> for Event {
    fn from(event: kad::Event) -> Self {
        Event::Kademlia(event)
    }
}

impl From<identify::Event> for Event {
    fn from(identify_event: identify::Event) -> Self {
        Event::Identify(identify_event)
    }
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