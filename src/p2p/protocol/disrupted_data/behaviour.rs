use std::iter;
use std::time::Duration;

use crate::p2p::protocol::disrupted_data::request::FindResult;
use crate::p2p::protocol::disrupted_data::request::Request;
use crate::p2p::protocol::disrupted_data::request::Request::GET;
use crate::p2p::protocol::disrupted_data::request::Request::PUT;
use crate::p2p::protocol::disrupted_data::types::state::{GetRequestState, PutRequestState};
use disrupted_data_sdk_rs::{ActionResult, Actions};
use libp2p::identity::Keypair;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode::Server;
use libp2p::kad::{Record, RecordKey};
use libp2p::request_response::{json, InboundRequestId, ProtocolSupport, ResponseChannel};
use libp2p::swarm::{ConnectionHandler, NetworkBehaviour};
use libp2p::{identify, kad, ping, request_response, PeerId, StreamProtocol};
use sha2::digest::Mac;

pub(crate) type RequestResponseBehaviour = json::Behaviour<Actions, ActionResult>;
pub(crate) type RequestResponseEvent = request_response::Event<Actions, ActionResult>;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct Behaviour {
	pub(crate) request_response: json::Behaviour<Actions, ActionResult>,
	pub(crate) kad: kad::Behaviour<MemoryStore>,
	pub(crate) identify: identify::Behaviour,
}

impl Behaviour {
	pub fn new(peer_id: &PeerId, key_pair: &Keypair) -> Self {
		let store = MemoryStore::new(*peer_id);
		let mut kad = kad::Behaviour::new(*peer_id, store);
		kad.set_mode(Option::from(Server));

		let protocol = StreamProtocol::new("/disrupted-data/browser/1");
		let protocols = iter::once((protocol, ProtocolSupport::Full));
		let request_response = RequestResponseBehaviour::new(protocols, request_response::Config::default().with_request_timeout(Duration::new(30, 0)));


		let ping = ping::Behaviour::new(Default::default());

		let identify_config = identify::Config::new(
			"/ipfs/id/1.0.0".to_string(),
			key_pair.public(),
		).with_interval(Duration::from_secs(20));

		let identify = identify::Behaviour::new(identify_config);

		Behaviour {
			request_response,
			kad,
			identify,
			// ping,
		}
	}


	pub(crate) fn put(&mut self, peer: PeerId, request_id: InboundRequestId, request: Actions) -> Option<Request> {
		let inbound_request: Request = (request_id, request).try_into().unwrap();
		// println!("Handling disrupted_data message in verify state in behaviour: {:?}", inbound_request);
		let verification_result = inbound_request.verify_request();
		let user_key = RecordKey::new(&inbound_request.clone().get_user_public_key());
		match &inbound_request {
			PUT(put_request_state, _) => {
				let next_state = put_request_state.verify(&mut self.kad, verification_result, user_key);
				// println!("Next state obtained in behaviour: {:?}", next_state);
				Some(PUT(next_state.clone(), inbound_request.get_data()))
			}
			GET(get_request_state, _) => { None }
		}
	}

	pub(crate) fn get_query_progress(&mut self, find_result: FindResult, possible_user_record: Option<Record>, request: Request) -> Option<Request> {
		// println!("Handling get_query_progress in behaviour: {:?} for inbound request id: {:?} find result: {:?}", request, inbound_request_id, find_result);
		match &request {
			PUT(put_request_state, data) => {
				let updated_state_with_find_result = put_request_state.find_user_result(find_result);
				// println!("updated state with find result in get_query_progress behaviour: {:?}", updated_state_with_find_result);
				let next_state = updated_state_with_find_result.create_data_record(&mut self.kad, data.clone());

				// println!("Next state obtained in get_query_progress behaviour: {:?}", next_state);
				Some(PUT(next_state.clone(), data.clone()))
			}
			GET(_, _) => { None }
		}
	}

	pub(crate) fn put_data_query_progress(&mut self, request: Request) -> Option<Request> {
		match &request {
			PUT(request_state, data) => {
				// println!("WaitingDataCreate in put_query_progress with state:{:?} and data: {:?}", request_state, data);
				if let PutRequestState::WaitingDataCreate(_, _) = &request_state {
					let possible_waiting_user_create_state = request_state.create_or_update_user(&mut self.kad, data.clone());

					Some(PUT(possible_waiting_user_create_state.clone(), data.clone()))
				} else { None }
			}
			GET(_, _) => { None }
		}
	}
	pub(crate) fn put_user_query_progress(&mut self, request: Request, channel: ResponseChannel<ActionResult>) {
		match &request {
			PUT(request_state, data) => {
				// println!("WaitingUserCreate in put_query_progress with state:{:?} and data: {:?}", request_state, data);
				if let PutRequestState::WaitingUserCreate(query_id) = &request_state {
					let send_result = self.request_response.send_response(channel, ActionResult::Success("Data added".to_string()));
					// println!("SendResult in WaitingUserCreate put_query_progress behaviour: {:?}", send_result);
				}
			}
			GET(_, _) => {}
		}
	}
	pub(crate) fn send_response(&mut self, action_result: ActionResult, channel: ResponseChannel<ActionResult>) {
		let send_result = self.request_response.send_response(channel, action_result);
		// println!("Sending response result in behaviour: {:?}", send_result);
	}

	pub(crate) fn get(&mut self, peer: PeerId, request_id: InboundRequestId, request: Actions) -> Option<Request> {
		let inbound_request: Request = (request_id, request).try_into().unwrap();
		// println!("Handling disrupted_data message in GET verify state in behaviour: {:?}", inbound_request);
		let verification_result = inbound_request.verify_request();
		let user_key = RecordKey::new(&inbound_request.clone().get_user_public_key());
		match &inbound_request {
			GET(get_request_state, _) => {
				let next_state = get_request_state.verify(&mut self.kad, verification_result, user_key);
				// println!("Next state obtained in behaviour after verify: {:?}", next_state);
				Some(GET(next_state.clone(), inbound_request.get_data()))
			}
			PUT(_, _) => { None }
		}
	}

	pub(crate) fn find_user_query_progress(&mut self, request_id: InboundRequestId, find_result: FindResult, request: Request) -> GetRequestState {
		match &request {
			GET(get_request_state, data) => {
				let updated_state_with_find_result = get_request_state.find_user_result(find_result);
				// println!("updated state with find result in find_user_query_progress behaviour: {:?}", updated_state_with_find_result);
				if let GetRequestState::FindData(user) = &updated_state_with_find_result {
					if user.contains_data_record_key(data.clone().record_key) {
						updated_state_with_find_result.clone()
					} else {
						// self.send_response(ActionResult::Failure("Data not associated with user".to_string()), channel);
						GetRequestState::SendResponse(ActionResult::Failure("Data not associated with user".to_string()))
					}
				} else if let GetRequestState::UserNotFound = &updated_state_with_find_result {
					// self.send_response(ActionResult::Failure("User not found".to_string()), channel);
					GetRequestState::SendResponse(ActionResult::Failure("User not found".to_string()))
				} else { GetRequestState::Invalid }
			}
			PUT(_, _) => { GetRequestState::Invalid }
		}
	}

	pub(crate) fn find_data(&mut self, request: Request) -> Option<Request> {
		if let GET(request_state, data) = request {
			match &request_state {
				GetRequestState::FindData(user) => {
					let next_state = request_state.find_data_record(&mut self.kad, data.clone());
					// println!("Next state obtained in behaviour after find user record: {:?}", next_state);
					Some(GET(next_state.clone(), data.clone()))
				}
				_ => { None }
			}
		} else { None }
	}

	pub(crate) fn find_data_query_progress(&mut self, find_result: FindResult, message: Option<String>, request: Request, channel: ResponseChannel<ActionResult>) {
		match &request {
			GET(get_request_state, data) => {
				let updated_state_with_find_result = get_request_state.find_data_result(find_result);
				// println!("updated state with find result in find_data_query_progress behaviour: {:?}", updated_state_with_find_result);
				if let GetRequestState::SendResponse(action_result) = &updated_state_with_find_result {
					self.send_response(action_result.clone(), channel);
				} else if let GetRequestState::CouldNotGetData = &updated_state_with_find_result {
					self.send_response(ActionResult::Failure(message.clone().unwrap()), channel);
				}
			}
			PUT(_, _) => {}
		}
	}
}

#[derive(Debug)]
pub enum Event {
	Kademlia(kad::Event),
	Identify(identify::Event),
	RequestResponse(RequestResponseEvent),
	VerificationResult(),
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
	fn from(request_response_event: RequestResponseEvent) -> Self {
		Event::RequestResponse(request_response_event)
	}
}