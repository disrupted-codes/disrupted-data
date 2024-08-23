use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use libp2p::kad;
use libp2p::kad::{QueryId, Record};
use libp2p::request_response::ResponseChannel;
use serde::{Deserialize, Serialize};

use disrupted_data_sdk_rs::{ActionResult, Actions, DisruptedDataError};
use disrupted_data_sdk_rs::ActionResult::{Failure, Success};

// use disrupted_data_sdk_rs::types::{get_secp256k1_public_key, is_identity_verified};
use crate::p2p::protocol::disrupted_data;
use crate::p2p::protocol::request_response::{DisruptedDataRecord, Sec256k1PublicKey, Sec256k1Signature};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Event {
    // Put(Put),
    // GET(Get),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum Group {
    Nostr,
    Git,
}

impl Event {
    pub(crate) fn process_request(behaviour: &mut disrupted_data::Behaviour, incoming_request: Actions, requests: &Arc<Mutex<HashMap<QueryId, ResponseChannel<ActionResult>>>>, channel: ResponseChannel<ActionResult>) {
        // pub(crate) fn process_request(behaviour: &mut disrupted_data::Behaviour, incoming_request: Actions, requests: &Arc<Mutex<HashMap<QueryId, ResponseChannel<ActionResult>>>>, channel: ResponseChannel<ActionResult>) -> Result<QueryId, DisruptedDataError> {
        println!("handling incoming request for: {:?}", incoming_request);
        match incoming_request {
            Actions::Put(put_request) => {
                println!("put record: {:?}", put_request.clone().to_record());
                // TODO verify signature
                let put_request_clone = put_request.clone();
                let parts: Vec<String> = vec![
                    put_request_clone.record_value
                ];

                // let secp256k1_public_key_result = get_secp256k1_public_key(put_request_clone.user_public_key);

                // match secp256k1_public_key_result {
                //     Ok(public_key) => {
                //             if is_identity_verified(put_request_clone.signature, public_key, get_message(parts)) {
                // TODO create tree

                //                     let kad_put_result = behaviour.kad.put_record(put_request.to_record(), Quorum::One);
                //
                //                     match kad_put_result {
                //                         Ok(query_id) => {
                //                             println!("Query ID for PUT: {:?}", query_id);
                //
                //                             let mut requests_guard = requests.lock().unwrap();
                //                             requests_guard.insert(query_id, channel);
                //                             Ok(query_id)
                //                         }
                //                         Err(store_error) => {
                //                             let disrupted_data_error = Self::send_fail_response(behaviour, channel, store_error.to_string());
                //                             Err(disrupted_data_error)
                //                         }
                //                     }
                //                 } else {
                //                     println!("Could not put record");
                //                     let error_message = "Identity could not be verified".to_string();
                //                     let disrupted_data_error = Self::send_fail_response(behaviour, channel, error_message);
                //
                //                     Err(disrupted_data_error)
                //                 }
                //
                //         }
                //         Err(error) => {
                //             let disrupted_data_error = Self::send_fail_response(behaviour, channel, error.message);
                //             Err(disrupted_data_error)
                //         }
                //     }
                //
            }
            Actions::Get(get_request) => {
                //     let get_request_clone = get_request.clone();
                //     let secp256k1_public_key_result = get_secp256k1_public_key(get_request_clone.user_public_key);
                //
                //     match secp256k1_public_key_result {
                //         Ok(secp256k1_key) => {
                //             if is_identity_verified(get_request_clone.signature, secp256k1_key, get_message(vec![get_request_clone.record_key])) {
                //                 let query_id = behaviour.kad.get_record(get_request.to_record_key());
                //
                //                 println!("Query ID for GET: {:?}", query_id);
                //                 let mut requests_guard = requests.lock().unwrap();
                //                 requests_guard.insert(query_id, channel);
                //
                //                 Ok(query_id)
                //             } else {
                //                 let error_message = "Identity could not be verified".to_string();
                //                 let disrupted_data_error = Self::send_fail_response(behaviour, channel, error_message);
                //
                //                 Err(disrupted_data_error)
                //             }
                //
                //
                //         }
                //         Err(error) => {
                //             let disrupted_data_error = Self::send_fail_response(behaviour, channel, error.message);
                //             Err(disrupted_data_error)
                //         }
                //     }
                //
            }
            Actions::Unknown => {}
        }
    }
}

// fn send_fail_response(behaviour: &mut disrupted_data::Behaviour, channel: ResponseChannel<ActionResult>, error_message: String) -> DisruptedDataError {
//     let disrupted_data_error = DisruptedDataError { message: error_message };
//     behaviour.request_response.send_response(channel, Failure(disrupted_data_error.message.clone())).unwrap();
//     disrupted_data_error
// }
//
// pub(crate) fn send_get_response(behaviour: &mut disrupted_data::Behaviour, query_id: &QueryId, record: Record, requests: &Arc<Mutex<HashMap<QueryId, ResponseChannel<ActionResult>>>>) {
//     let mut requests_guard = requests.lock().unwrap();
//     let response_channel_result = requests_guard.remove(&query_id);
//     match response_channel_result {
//         None => {
//             println!("Response channel for GET Not found");
//         }
//         Some(response_channel) => {
//             println!("Response channel for GET Query ID: {:?}", query_id);
//             behaviour.request_response.send_response(response_channel, Success(String::from_utf8(record.value).unwrap())).unwrap()
//         }
//     }
// }
//
// pub(crate) fn send_put_response(behaviour: &mut disrupted_data::Behaviour, query_id: &QueryId, requests: &Arc<Mutex<HashMap<QueryId, ResponseChannel<ActionResult>>>>, put_result: Result<(kad::RecordKey), DisruptedDataError>) {
//     let mut requests_guard = requests.lock().unwrap();
//     let response_channel_result = requests_guard.remove(&query_id);
//     match response_channel_result {
//         None => {
//             println!("Response channel for PUT Not found");
//         }
//         Some(response_channel) => {
//             match put_result {
//                 Ok(key) => {
//                     println!("Response channel for PUT Query ID: {:?}", query_id);
//                     behaviour.request_response.send_response(response_channel, Success(String::from_utf8(key.to_vec()).unwrap())).unwrap();
//                 }
//                 Err(error) => {}
//             }
//         }
//     }
// }




// #[derive(Debug, Clone, Eq, PartialEq)]
// pub(crate) struct Put {
//     k: Sec256k1PublicKey,
//     r: DisruptedDataRecord,
//     g: Option<Group>,
//     s: Sec256k1Signature,
// }
//
// #[derive(Debug, Clone, Eq, PartialEq)]
// pub(crate) struct Get {
//     k: Sec256k1PublicKey,
// }
