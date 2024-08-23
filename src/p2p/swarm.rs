use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use libp2p::{identify, kad, Multiaddr, noise, request_response, Swarm, swarm, tcp, Transport, yamux};
use libp2p::core::upgrade::Version;
use libp2p::futures::{FutureExt, StreamExt};
use libp2p::kad::{QueryId, QueryResult, Quorum};
use libp2p::request_response::{InboundRequestId, Message, ResponseChannel};
use libp2p::swarm::SwarmEvent;
use tokio::sync::mpsc::{Receiver, Sender};
use toml::Table;

use disrupted_data_sdk_rs::ActionResult;

use crate::p2p::{FromDisruptedDataSwarmEvent, ToDisruptedDataSwarmEvent, User};
use crate::p2p::FromDisruptedDataSwarmEvent::{NewRequest, UserFound, UserNotFound};
use crate::p2p::node::Node;
use crate::p2p::protocol::disrupted_data;
use crate::p2p::ToDisruptedDataSwarmEvent::{Get, GetUser, Put, PutUser, SendResponse};
use crate::types::NodeConfig;

pub struct DisruptedDataSwarm {
    node: Node,
    swarm: Swarm<disrupted_data::Behaviour>,
    swarm_event_sender: Sender<FromDisruptedDataSwarmEvent>,
    request_event_receiver: Receiver<ToDisruptedDataSwarmEvent>,
    requests: Arc<Mutex<HashMap<InboundRequestId, ResponseChannel<ActionResult>>>>,
    get_queries: Arc<Mutex<HashMap<QueryId, ToDisruptedDataSwarmEvent>>>,
    put_requests: Arc<Mutex<HashMap<QueryId, (Option<User>, InboundRequestId)>>>,

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
            requests: Arc::new(Mutex::new(HashMap::new())),
            get_queries: Arc::new(Mutex::new(HashMap::new())),
            put_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }


    fn init_swarm(node: &Node, bootstrap_nodes: Table) -> Swarm<disrupted_data::Behaviour> {
        let peer_id = &node.peer_id;
        let keypair = &node.key;
        let behaviour = disrupted_data::Behaviour::new(peer_id, &keypair);

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
                Some(request_event) = self.request_event_receiver.recv() => {
                    match request_event {
                        PutUser(inbound_request_id, record) => {
                            println!("processing put user");

                            let query_id = self.swarm.behaviour_mut().kad.put_record(record.clone(), Quorum::One).expect("KAD error: Unable to put record");
                            let mut put_requests_guard = self.put_requests.lock().unwrap();
                            put_requests_guard.insert(query_id, (None, inbound_request_id));
                        },
                        Put(inbound_request_id, record) => {
                            println!("processing Put data");
                            let user = record.0.clone();
                            let query_id = self.swarm.behaviour_mut().kad.put_record(record.1.clone(), Quorum::One).expect("KAD error: Unable to put record");
                            let mut put_requests = self.put_requests.lock().unwrap();

                            put_requests.insert(query_id, (Some(user), inbound_request_id));

                        },
                        GetUser(inbound_request_id, record_key) => {
                            println!("Processing get user");
                            let query_id = self.swarm.behaviour_mut().kad.get_record(record_key.clone());
                            let mut get_queries_mutex = self.get_queries.lock().unwrap();
                            get_queries_mutex.insert(query_id, GetUser(inbound_request_id, record_key.clone()));

                        },
                        Get(inbound_request_id, record_key) => {
                            println!("Processing get record");
                            let query_id = self.swarm.behaviour_mut().kad.get_record(record_key.clone());
                            let mut get_queries_mutex = self.get_queries.lock().unwrap();
                            get_queries_mutex.insert(query_id, Get(inbound_request_id, record_key.clone()));

                        },
                        SendResponse(inbound_request_id, action_result) => {
                            let mut requests_guard = self.requests.lock().unwrap();
                            if let Some(response_channel) = requests_guard.remove(&inbound_request_id) {
                                self.swarm.behaviour_mut().request_response.send_response(response_channel, action_result).unwrap();
                            }
                        }
                    }
                }
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::ConnectionEstablished {peer_id, ..} => {
                            println!("Established connection with peer ID: {}", peer_id)
                        },
                        SwarmEvent::Behaviour(disrupted_data::Event::Kademlia (kad::Event::RoutingUpdated {is_new_peer, ..})) => {
                            self.swarm.behaviour_mut().kad.bootstrap().unwrap();
                            println!("Routing updated - bootstrapping")

                        },
                        SwarmEvent::Behaviour(disrupted_data::Event::RequestResponse(request_response::Event::Message {message,..} ) )=> {
                            println!("Received request response message: {:?}", message);
                            match message {
                                Message::Request{ request_id, request, mut channel } => {
                                    let get_queries_guard = self.get_queries.lock().unwrap();
                                    let mut requests_guard = self.requests.lock().unwrap();
                                    requests_guard.insert(request_id, channel);
                                    self.swarm_event_sender.send(NewRequest(request_id, request.clone())).await.unwrap();

                                },
                                Message::Response{ request_id, response } => {
                                    println!("request ID: {} has response: {:?}", request_id, response)
                                }
                            }
                        },
                        SwarmEvent::Behaviour(disrupted_data::Event::Kademlia(kad::Event::OutboundQueryProgressed {id, result, ..})) => {
                            match result {
                                QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(kad::PeerRecord {record, peer}))) => {
                                    println!("GetRecordOK receieved for query: {:?}", id);
                                    let mut get_queries_guard = self.get_queries.lock().unwrap();
                                    if let Some(to_swarm_event) = get_queries_guard.remove(&id){
                                        if let GetUser(inbound_request_id, user_key) = to_swarm_event.clone() {
                                            self.swarm_event_sender.send(UserFound(inbound_request_id.clone(), record.clone().try_into().unwrap())).await.unwrap()
                                        }
                                        if let Get(inbound_request_id, user_key) = to_swarm_event.clone() {
                                            let mut requests_guard = self.requests.lock().unwrap();
                                            if let Some(response_channel) = requests_guard.remove(&inbound_request_id) {
                                                self.swarm.behaviour_mut().request_response.send_response(response_channel, ActionResult::Success(String::from_utf8(record.clone().value).unwrap())).unwrap();
                                            }
                                        }
                                    }
                                },
                                QueryResult::GetRecord(Err(kad::GetRecordError::NotFound {key, ..})) => {
                                    println!("Record not found for key: {:?}", key);

                                    let mut requests_guard = self.requests.lock().unwrap();
                                    let mut get_queries_guard = self.get_queries.lock().unwrap();

                                    if let Some(to_swarm_event) = get_queries_guard.remove(&id){
                                        if let GetUser(inbound_request_id, user_key) = to_swarm_event {
                                            let new_user = User::new(&user_key);
                                            self.swarm_event_sender.send(UserNotFound(inbound_request_id.clone(), new_user)).await.unwrap()
                                        }
                                    }

                                }
                                QueryResult::PutRecord(Ok(kad::PutRecordOk{key})) => {
                                    eprintln!("Successfully PUT key: {:?} ", key);
                                    let mut requests_guard = self.requests.lock().unwrap();
                                    let mut put_requests_guard = self.put_requests.lock().unwrap();

                                    if let Some((user, inbound_request_id)) = put_requests_guard.remove(&id) {
                                        if let Some(response_channel) = requests_guard.remove(&inbound_request_id) {
                                            match user {
                                                //TODO Hack! We are putting data if User exists, if user is None, it means data has been created and the user record is being updated.
                                                Some(user) => {
                                                    self.swarm_event_sender.send(FromDisruptedDataSwarmEvent::PutDataSuccess(inbound_request_id)).await.unwrap();
                                                    requests_guard.insert(inbound_request_id, response_channel);
                                                    put_requests_guard.insert(id, (None, inbound_request_id));
                                                },
                                                None => {
                                                    self.swarm_event_sender.send(FromDisruptedDataSwarmEvent::PutUserSuccess).await.unwrap();
                                                    self.swarm.behaviour_mut().request_response.send_response(response_channel, ActionResult::Success(String::from_utf8(key.to_vec()).unwrap())).unwrap();

                                                }
                                            }
                                        }
                                    }
                                },
                                _ => {eprintln!("outbound query result: {:?}", result)}
                            }

                        }
                        SwarmEvent::Behaviour( disrupted_data::Event::Identify(identify::Event::Received { peer_id ,info })) => {
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