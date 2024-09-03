use std::collections::HashMap;

use hex::encode;
use libp2p::kad::{Record, RecordKey};
use libp2p::request_response::InboundRequestId;
use tokio::sync::mpsc;

use disrupted_data_sdk_rs::{ActionResult, Actions};

use crate::p2p::{FromDisruptedDataSwarmEvent, ToDisruptedDataSwarmEvent};
use crate::p2p::ToDisruptedDataSwarmEvent::{Get, GetUser, Put, PutUser, SendResponse};

pub struct RequestHandler {
    from_swarm_event_receiver: mpsc::Receiver<FromDisruptedDataSwarmEvent>,
    to_swarm_event_sender: mpsc::Sender<ToDisruptedDataSwarmEvent>,
    put_requests: HashMap<InboundRequestId, FromDisruptedDataSwarmEvent>,
    get_requests: HashMap<InboundRequestId, FromDisruptedDataSwarmEvent>,
}

impl RequestHandler {
    pub fn new(mut from_swarm_event_receiver: mpsc::Receiver<FromDisruptedDataSwarmEvent>,
               to_swarm_event_sender: mpsc::Sender<ToDisruptedDataSwarmEvent>,
    ) -> Self {
        Self {
            from_swarm_event_receiver,
            to_swarm_event_sender,
            put_requests: HashMap::new(),
            get_requests: HashMap::new(),
        }
    }
    pub async fn process(mut self) {
        loop {
            match self.from_swarm_event_receiver.recv().await {
                Some(FromDisruptedDataSwarmEvent::NewRequest(inbound_request_id, action)) => {
                    println!("Handling new request");
                    let identity_verification_result = action.clone().verify_identity();
                    let action_clone = action.clone();
                    match identity_verification_result {
                        Ok(_) => {
                            match action {
                                Actions::Put(put_request) => {
                                    let record_key = RecordKey::from(put_request.clone().user_public_key);

                                    self.to_swarm_event_sender.send(GetUser(inbound_request_id, record_key.clone())).await.unwrap();
                                    self.put_requests.insert(inbound_request_id, FromDisruptedDataSwarmEvent::NewRequest(inbound_request_id, action_clone));
                                }
                                Actions::Get(get_request) => {
                                    let record_key = RecordKey::from(get_request.clone().user_public_key);
                                    self.to_swarm_event_sender.send(GetUser(inbound_request_id, record_key.clone())).await.unwrap();

                                    self.get_requests.insert(inbound_request_id, FromDisruptedDataSwarmEvent::NewGetRequest(inbound_request_id, action_clone));
                                }
                                Actions::Unknown => {}
                            }
                        }
                        Err(error) => {}
                    }
                }
                Some(FromDisruptedDataSwarmEvent::UserFound(inbound_request_id, mut user)) => {
                    let optional_previous_put_event = self.put_requests.remove(&inbound_request_id);
                    match optional_previous_put_event {
                        None => {
                            let optional_previous_get_event = self.get_requests.remove(&inbound_request_id);
                            match optional_previous_get_event {
                                None => {
                                    println!("No associated previous events found for inbound request ID: {:?}. ", inbound_request_id)
                                }
                                Some(previous_get_event) => {
                                    if let FromDisruptedDataSwarmEvent::NewGetRequest(inbound_request_id, action) = previous_get_event {
                                        let get_record_key_hex = action.clone().get_record_key_hex().unwrap();
                                        println!("get_record_key_hex: {}", get_record_key_hex.clone());
                                        let existing_data_records = user.data_record_keys;
                                        for existing_data_record in existing_data_records.clone() {
                                            println!("existing data record: {}", existing_data_record.clone());
                                        }
                                        if existing_data_records.contains(&get_record_key_hex) {
                                            self.to_swarm_event_sender.send(Get(inbound_request_id, action.clone().get_record_key().unwrap())).await.unwrap();
                                        } else {
                                            self.to_swarm_event_sender.send(SendResponse(inbound_request_id, ActionResult::Failure("Data not found for user".to_string()))).await.unwrap();
                                        }
                                    }
                                }
                            }
                        }
                        Some(previous_put_event) => {
                            if let FromDisruptedDataSwarmEvent::NewRequest(inbound_request_id, action) = previous_put_event {
                                let record = action.clone().get_record().unwrap();
                                let new_record_key_hex = String::from_utf8(record.clone().key.to_vec()).unwrap();
                                // let new_record_key_hex = encode(record.clone().key);
                                println!("PUT New record key: {}", new_record_key_hex);

                                let record_key = RecordKey::new(&new_record_key_hex.clone());
                                let new_data_record = Record::new(record_key.clone(), record.clone().value);
                                let mut updated_user = user.clone();
                                updated_user.add_data_record_keys(new_record_key_hex);

                                self.to_swarm_event_sender.send(Put(inbound_request_id, (updated_user.clone(), new_data_record))).await.unwrap();
                                self.put_requests.insert(inbound_request_id, FromDisruptedDataSwarmEvent::UserFound(inbound_request_id, updated_user.clone()));
                            }
                        }
                    }
                }
                Some(FromDisruptedDataSwarmEvent::UserNotFound(inbound_request_id, mut new_user)) => {
                    let previous_event_option = self.put_requests.remove(&inbound_request_id);
                    match previous_event_option {
                        None => {
                            let get_request_option = self.get_requests.remove(&inbound_request_id);
                            match get_request_option {
                                None => {
                                    println!("Could not find get response channel");
                                    self.to_swarm_event_sender.send(SendResponse(inbound_request_id, ActionResult::Failure("System error: could not find get response channel".to_string()))).await.unwrap();
                                }
                                Some(get_request) => {
                                    println!("User not found - get request not found??");
                                    self.to_swarm_event_sender.send(SendResponse(inbound_request_id, ActionResult::Failure("Data not found for user".to_string()))).await.unwrap();
                                }
                            }
                        }
                        Some(previous_event) => {
                            if let FromDisruptedDataSwarmEvent::NewRequest(inbound_request_id, action) = previous_event {

                                let record = action.clone().get_record().unwrap();
                                let record_key_hex = String::from_utf8(record.clone().key.to_vec()).unwrap();
                                // let new_record_key_hex = encode(record.clone().key);
                                println!("PUT New record key: {}", record_key_hex);

                                // let record = action.clone().get_record().unwrap();
                                // let record_key_hex = encode(record.clone().key);

                                let record_key = RecordKey::new(&record_key_hex.clone());
                                let data_record = Record::new(record_key.clone(), record.clone().value);
                                new_user.add_data_record_keys(record_key_hex);

                                self.put_requests.insert(inbound_request_id, FromDisruptedDataSwarmEvent::UserNotFound(inbound_request_id, new_user.clone()));
                                self.to_swarm_event_sender.send(Put(inbound_request_id, (new_user.clone(), data_record))).await.unwrap()
                            }

                        }
                    }
                }
                Some(FromDisruptedDataSwarmEvent::PutUserSuccess) => {
                    println!("User successfully updated");
                }
                Some(FromDisruptedDataSwarmEvent::PutUserFail(user_key, query_id)) => {}
                Some(FromDisruptedDataSwarmEvent::PutDataSuccess(inbound_request_id)) => {
                    let optional_previous_event = self.put_requests.remove(&inbound_request_id);
                    match optional_previous_event {
                        None => {
                            println!("previous event not found");
                        }
                        Some(previous_event) => {
                            if let FromDisruptedDataSwarmEvent::UserNotFound(inbound_request_id, user) = previous_event.clone() {
                                self.to_swarm_event_sender.send(PutUser(inbound_request_id, user.try_into().unwrap())).await.unwrap()
                            }
                            if let FromDisruptedDataSwarmEvent::UserFound(inbound_request_id, user) = previous_event.clone() {
                                self.to_swarm_event_sender.send(PutUser(inbound_request_id, user.try_into().unwrap())).await.unwrap()
                            }
                        }
                    }
                    //TODO If User was found in the previous step, we need to update the user record with the added data_record_key here
                    //TODO we still need to call PutUser, so what is the use of UserNotFound and UserFound events?
                }
                Some(FromDisruptedDataSwarmEvent::PutDataFail(query_id)) => {}

                _ => {}
            }
        }
    }
}


