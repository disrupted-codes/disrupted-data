use libp2p::kad::{Record, RecordKey};
use libp2p::request_response::InboundRequestId;
use serde::{Deserialize, Serialize};

use disrupted_data_sdk_rs::{ActionResult, Actions, DisruptedDataError};
pub use swarm::DisruptedDataSwarm;

pub mod node;
mod protocol;
mod swarm;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromDisruptedDataSwarmEvent {
    NewRequest(InboundRequestId, Actions),
    NewGetRequest(InboundRequestId, Actions),
    UserFound(InboundRequestId, User),
    UserNotFound(InboundRequestId, User),
    PutUserSuccess,
    PutUserFail(String, String),
    PutDataSuccess(InboundRequestId),
    PutDataFail(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToDisruptedDataSwarmEvent {
    PutUser(InboundRequestId, Record),
    Put(InboundRequestId, (User, Record)),
    GetUser(InboundRequestId, RecordKey),
    Get(InboundRequestId, RecordKey),
    SendResponse(InboundRequestId, ActionResult),

}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub key: String,
    pub data_record_keys: Vec<String>,
}

impl User {
    pub fn new(key: &RecordKey) -> Self {
        Self {
            key: String::from_utf8(key.to_vec()).expect("Invalid key when creating new user"),
            data_record_keys: vec![],
        }
    }
    pub fn add_data_record_keys(&mut self, data_record_key: String) {
        self.data_record_keys.push(data_record_key)
    }

    pub(crate) fn contains_data_record_key(&self, key: String) -> bool {
        self.data_record_keys.contains(&key)
    }
}

impl From<Record> for User {
    // fn from(value: (Vec<u8>, Record)) -> Self {
    //     Self {
    //         key: String::from_utf8(value[1].record.key.to_vec()).expect("Invalid record key"),
    //         data_record_keys: split_raw_data_record_keys(value[0], value[1].record.value),
    //     }
    // }
    fn from(record: Record) -> Self {
        Self{
            key: String::from_utf8(record.key.to_vec()).expect("Invalid user key"),
            data_record_keys: split_raw_data_record_keys(record.value)

        }
    }
}

impl TryInto<Record> for User {
    type Error = DisruptedDataError;

    fn try_into(self) -> Result<Record, Self::Error> {
        let raw_data_record_keys = join_data_record_keys(self.data_record_keys);

        let record_key = RecordKey::from(self.key.into_bytes());
        Ok(Record::new(record_key, raw_data_record_keys))
    }
}

fn join_data_record_keys(data_record_keys: Vec<String>) -> Vec<u8> {
    data_record_keys.join("|").into_bytes()
}

fn split_raw_data_record_keys( raw_data_record_keys: Vec<u8>) -> Vec<String> {
    let comma_delimited_data_record_keys = String::from_utf8(raw_data_record_keys).expect("Invalid data record keys");
    comma_delimited_data_record_keys.split('|').map(|data_record_key| { data_record_key.to_string() }).collect()
}