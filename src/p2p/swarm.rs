use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use crate::p2p::node::Node;
use crate::p2p::protocol::disrupted_data;
use crate::p2p::protocol::disrupted_data::behaviour;
use crate::p2p::protocol::disrupted_data::request::Request::{GET, PUT};
use crate::p2p::protocol::disrupted_data::request::{FindResult, Request};
use crate::p2p::protocol::disrupted_data::types::state::{GetRequestState, PutRequestState};
use crate::p2p::{FromDisruptedDataSwarmEvent, ToDisruptedDataSwarmEvent};
use crate::types::NodeConfig;
use disrupted_data_sdk_rs::{ActionResult, Actions};
use libp2p::core::upgrade::Version;
use libp2p::futures::{FutureExt, StreamExt};
use libp2p::kad::{QueryId, QueryResult};
use libp2p::request_response::{InboundRequestId, Message, ResponseChannel};
use libp2p::swarm::SwarmEvent;
use libp2p::{identify, kad, noise, request_response, swarm, tcp, yamux, Multiaddr, Swarm, Transport};
use tokio::sync::mpsc::{Receiver, Sender};
use toml::Table;

pub(crate) type RequestsHashMap = HashMap<InboundRequestId, (Request, ResponseChannel<ActionResult>)>;

pub struct DisruptedDataSwarm {
	node: Node,
	swarm: Swarm<behaviour::Behaviour>,
	swarm_event_sender: Sender<FromDisruptedDataSwarmEvent>,
	request_event_receiver: Receiver<ToDisruptedDataSwarmEvent>,
	requests: RequestsHashMap,
	kad_request_mapping: HashMap<QueryId, InboundRequestId>,

}

impl DisruptedDataSwarm {
	pub fn new(node_config: NodeConfig, swarm_event_sender: Sender<FromDisruptedDataSwarmEvent>, mut request_event_receiver: Receiver<ToDisruptedDataSwarmEvent>) -> Self {
		let bootstrap_nodes = node_config.clone().bootstrap_nodes();
		let node = Node::new(node_config);

		Self {
			node: node.clone(),
			swarm: Self::init_swarm(&node, bootstrap_nodes),
			swarm_event_sender,
			request_event_receiver,
			requests: HashMap::new(),
			kad_request_mapping: HashMap::new(),
		}
	}


	fn init_swarm(node: &Node, bootstrap_nodes: Table) -> Swarm<behaviour::Behaviour> {
		let peer_id = &node.peer_id;
		let keypair = &node.key;
		let behaviour = behaviour::Behaviour::new(peer_id, &keypair);

		let transport = tcp::tokio::Transport::default().upgrade(Version::V1).authenticate(noise::Config::new(&keypair).expect("Signing noise keypair")).multiplex(yamux::Config::default()).boxed();
		let swarm_config = swarm::Config::with_tokio_executor().with_idle_connection_timeout(Duration::from_secs(60));
		let mut swarm = Swarm::new(transport, behaviour, *peer_id, swarm_config);


		let tcp_address: Multiaddr = format!("/ip4/{}/tcp/{}", node.ip_address, node.port).parse().unwrap();

		swarm.listen_on(tcp_address).expect("Could not start listener");
		for (bootstrap_node_peer_id, bootstrap_node_ip) in &bootstrap_nodes {
			println!("Adding bootstrap node {} with IP {}", bootstrap_node_peer_id, bootstrap_node_ip.to_string());
			let bootstrap_multi_address = format!("/ip4/{}/tcp/6969", bootstrap_node_ip.as_str().unwrap());
			swarm.behaviour_mut().kad.add_address(&bootstrap_node_peer_id.parse().expect("Unable to parse peer id"),
			                                      bootstrap_multi_address.parse().expect("Unable to parse address"));
		}

		swarm
	}
	pub async fn start(&mut self) {
		loop {
			tokio::select! {
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::ConnectionEstablished {peer_id, ..} => {
                            println!("Established connection with peer ID: {}", peer_id)
                        },
                        SwarmEvent::Behaviour(behaviour::Event::Kademlia (kad::Event::RoutingUpdated {is_new_peer, ..})) => {
                            self.swarm.behaviour_mut().kad.bootstrap().unwrap();
                            println!("Routing updated - bootstrapping")

                        },
                        SwarmEvent::Behaviour(disrupted_data::behaviour::Event::RequestResponse(request_response::Event::Message {message, peer} ) )=> {
                            println!("Received request response message: {:?}", message);
                            match message {
                                Message::Request{ request_id, request, mut channel } => {
									match &request {
										Actions::Put(_) => {
		                                    let updated_request = self.swarm.behaviour_mut().put(peer, request_id, request);
		                                    match &updated_request {
		                                        None => {}
		                                        Some(request) => {
		                                            if let PUT(put_request_state, data) = request {
		                                                // println!("PUT Request after verification in swarm: {:?}", request);
														if let PutRequestState::FindUser(query_id) = put_request_state {
															self.kad_request_mapping.insert(query_id.clone(), request_id);
															self.requests.insert(request_id, (request.clone(), channel));
														} else if let PutRequestState::SendResponse(action_result) = put_request_state {
															self.swarm.behaviour_mut().send_response(action_result.clone(), channel)
														}
		                                            }

		                                        }
		                                    }
										}
										Actions::Get(_) => {
		                                    let updated_request = self.swarm.behaviour_mut().get(peer, request_id, request);
		                                    match &updated_request {
		                                        None => {}
		                                        Some(request) => {
		                                            if let GET(get_request_state, data) = request {
		                                                // println!("GET Request after verification in swarm: {:?}", request);
														if let GetRequestState::FindUser(query_id) = get_request_state {
															self.kad_request_mapping.insert(query_id.clone(), request_id);
															self.requests.insert(request_id, (request.clone(), channel));
														} else if let GetRequestState::SendResponse(action_result) = get_request_state {
															self.swarm.behaviour_mut().send_response(action_result.clone(), channel)
														}
		                                            }

		                                        }
		                                    }
										}
										Actions::Unknown => {}
									}

                                },
                                Message::Response{ request_id, response } => {
                                    // println!("request ID: {} has response: {:?}", request_id, response)
                                }
                            }
                        },
                        SwarmEvent::Behaviour(behaviour::Event::Kademlia(kad::Event::OutboundQueryProgressed {id, result, ..})) => {
                            match result {
                                QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(kad::PeerRecord {record, peer}))) => {
                                    // println!("GetRecordOK received for query: {:?} record: {:?}", id, record.clone());
									let inbound_request_id = self.kad_request_mapping.remove(&id);
									if inbound_request_id.is_some() {
										let request_and_channel = self.requests.remove(&inbound_request_id.unwrap()).unwrap();
										let request = request_and_channel.0;
										let channel = request_and_channel.1;
										match &request {
											PUT(_,_) => {
												let updated_request_state = self.swarm.behaviour_mut().get_query_progress(FindResult::Found(id, record.clone().try_into().unwrap()), Some(record), request);
												if let Some(PUT(PutRequestState::WaitingDataCreate(query_id, find_result), data)) = &updated_request_state {
													self.kad_request_mapping.insert(query_id.clone(), inbound_request_id.unwrap());
													self.requests.insert(inbound_request_id.unwrap(), (updated_request_state.unwrap().clone(), channel));

												} else if let Some(PUT(PutRequestState::SendResponse(action_result), data)) = &updated_request_state {
													let send_result = self.swarm.behaviour_mut().request_response.send_response(channel, action_result.clone());
													// println!("SendResponse result: {:?}", send_result);
												}
											}
											GET(get_request_state,data ) => {
												if let GetRequestState::FindUser(query_id) = &get_request_state {
													let updated_request_state = self.swarm.behaviour_mut().find_user_query_progress(inbound_request_id.unwrap(), FindResult::Found(query_id.clone(), record.clone()), request.clone());

													// println!("FindUser query progress result in swarm: {:?}", &updated_request_state);
													if let GetRequestState::FindData(user) = &updated_request_state {
														let updated_find_user_request = GET(updated_request_state.clone(), data.clone());
														let possible_waiting_data_request = self.swarm.behaviour_mut().find_data(updated_find_user_request.clone());
														// println!("Request state after calling behaviour find_data: {:?}", possible_waiting_data_request);
														if let Some(GET(GetRequestState::WaitingData(waiting_data_query_id), data)) = &possible_waiting_data_request {
															// println!("Adding query id {:?} in kad request mapping.", &waiting_data_query_id);
															self.kad_request_mapping.insert(waiting_data_query_id.clone(), inbound_request_id.unwrap());
															self.requests.insert(inbound_request_id.unwrap(), (possible_waiting_data_request.unwrap().clone(), channel));
														}
													} else if let GetRequestState::SendResponse(action_result) = updated_request_state {
														// println!("Sending response in swarm after find_user with action_result: {:?} on channel: {:?}", action_result, channel);
														let send_result = self.swarm.behaviour_mut().request_response.send_response(channel, action_result.clone());
														// println!("Error condition after processing find user - SendResponse result: {:?}", send_result);
													}
												} else if let GetRequestState::WaitingData(query_id) = &get_request_state {
													// println!("GetRequestState::WaitingData in GetRecordOK swarm for query id: {:?}", query_id);
													let action_result = ActionResult::Success(String::from_utf8(record.value).unwrap() );
													let send_result = self.swarm.behaviour_mut().request_response.send_response(channel, action_result.clone());
													// println!("Data find Success - SendResponse result : {:?}", send_result);

												}
											}
										}

									}


                                },
                                QueryResult::GetRecord(Err(kad::GetRecordError::NotFound {key, ..})) => {
                                    // println!("Record not found for key: {:?}", key);
									let inbound_request_id = self.kad_request_mapping.remove(&id);
									if inbound_request_id.is_some() {
										let request_and_channel = self.requests.remove(&inbound_request_id.unwrap()).unwrap();
										let request = request_and_channel.0;
										let channel = request_and_channel.1;
										match &request {
											PUT(_,_) => {
												let updated_request_state = self.swarm.behaviour_mut().get_query_progress(FindResult::NotFound, None, request);
												// println!("updated request state in GetRecord(NotFound()) swarm{:?}", updated_request_state);
												match &updated_request_state {
													None => {}
													Some(request) => {
														if let PUT(PutRequestState::WaitingDataCreate(data_put_query_id, find_result), data) = request {
															// println!("{:?} user_put_query_id WaitingDataCreate GetRecord(Err()) in swarm", data_put_query_id);
															self.requests.insert(inbound_request_id.unwrap(), (request.clone(), channel));
															// println!("{:?} current query mappings in WaitingDataCreate GetRecord(Err())swarm", self.kad_request_mapping);
															self.kad_request_mapping.insert(data_put_query_id.clone(), inbound_request_id.unwrap());

														}
													}
												}

											}
											GET(_,_) => {
												self.swarm.behaviour_mut().find_data_query_progress(FindResult::NotFound, Some("Record Not found".to_string()), request.clone(), channel);
											}
										}
									}

                                }
                                QueryResult::PutRecord(Ok(kad::PutRecordOk{key})) => {
                                    // eprintln!("Successfully PUT key: {:?} ", key);
									let inbound_request_id = self.kad_request_mapping.remove(&id);
									if inbound_request_id.is_some() {
										let request_and_channel = self.requests.remove(&inbound_request_id.unwrap()).unwrap();
										let put_request = request_and_channel.0;
										let channel = request_and_channel.1;
										// println!("Found put request in swarm: {:?}", put_request);
										match &put_request {
											PUT(put_request_state,data) => {
												if let PutRequestState::WaitingDataCreate(data_put_query_id, find_result) = put_request_state {
													let updated_request = self.swarm.behaviour_mut().put_data_query_progress(put_request);
													match &updated_request {
														None => {}
														Some(request) => {
															if let PUT(PutRequestState::WaitingUserCreate(user_put_query_id), data) = request {
																// println!("{:?} PutRecord(Ok()) in swarm", user_put_query_id);
																self.requests.insert(inbound_request_id.unwrap(), (request.clone(), channel));
																// println!("{:?} current query mappings inPutRecord(Ok()) swarm", self.kad_request_mapping);
																self.kad_request_mapping.insert(user_put_query_id.clone(), inbound_request_id.unwrap());
															}
														}
													}
												}else if let PutRequestState::WaitingUserCreate(user_put_query_id) = put_request_state {
													 self.swarm.behaviour_mut().put_user_query_progress(put_request, channel);
												}
											}
											GET(_,_) => {}
										}
									}


                                },
                                _ => {println!("outbound query result: {:?}", result)}
                            }

                        }
                        SwarmEvent::Behaviour( behaviour::Event::Identify(identify::Event::Received { peer_id ,info,.. })) => {
                            let ip_address = &self.node.ip_address;

                            let mut filtered_listening_address: Vec<Multiaddr> = info.listen_addrs.into_iter().filter(|address| !address.to_string().contains(ip_address) ).collect();
                            let mut listening_address = filtered_listening_address.remove(0);

                            self.swarm.behaviour_mut().kad.add_address(&peer_id, listening_address);
                        },
                        _ => {
                            println!("Received swarm event: {:?}", event)
                        }

                    }
                }
            }
		}
	}
}
fn read_file(path: PathBuf) -> Vec<u8> {
	let mut file = match File::open(&path) {
		Err(why) => panic!("Couldn't open {}: {}", path.display(), why.to_string()),
		Ok(file) => file,
	};

	let mut bytes = Vec::new();
	match file.read_to_end(&mut bytes) {
		Err(why) => panic!("Couldn't read {}: {}", path.display(), why.to_string()),
		Ok(_) => {}
	};

	bytes
}