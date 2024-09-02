use std::str::FromStr;

use hex::{decode, encode};
use libp2p::kad::{Record, RecordKey};
use serde::{Deserialize, Serialize};

use crate::{DisruptedDataError, get_message, Identity};
use crate::types::{get_secp256k1_public_key, is_identity_verified};
use crate::types::actions::Actions::Unknown;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Actions {
    Put(PutRequest),
    Get(GetRequest),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionResult {
    Success(String),
    Failure(String),
}

impl From<(String, &Identity)> for Actions {
    fn from(value: (String, &Identity)) -> Self {
        let (record_data, identity) = value;
        let parts: Vec<&str> = record_data.trim().split_whitespace().collect();
        if parts.len() < 2 {
            return Unknown;
        }

        let action = parts[0];
        let secp256k1_key_pair: libp2p::identity::secp256k1::Keypair = identity.keypair.clone().try_into_secp256k1().unwrap();
        let public_key = secp256k1_key_pair.public().clone();
        let secret_key = secp256k1_key_pair.secret().to_bytes().to_vec();

        match action.to_lowercase().as_str() {
            "put" => {
                let message_parts: Vec<String> = vec![
                    parts[1].to_string(),
                    parts[2].to_string(),
                ];
                let signature = Identity::sign(secret_key, parts[2].to_string());
                let hex_user_key = encode(public_key.to_bytes().to_vec());

                Actions::Put(PutRequest {
                    user_public_key: hex_user_key.into_bytes(),
                    record_key: parts[1].to_string(),
                    record_value: parts[2].to_string(),
                    signature,
                })
            }
            "get" => {
                let signature = Identity::sign(secret_key, parts[1].to_string());
                let hex_user_key = encode(public_key.to_bytes().to_vec());

                Actions::Get(GetRequest {
                    user_public_key: hex_user_key.into_bytes(),
                    record_key: parts[1].to_string(),
                    signature,
                })
            }
            _ => { Unknown }
        }
    }
}

impl Actions {
    pub fn verify_identity(self) -> Result<(), DisruptedDataError> {
        match self {
            Actions::Put(put_request) => {
                let public_key_bytes = decode(put_request.clone().user_public_key).expect("Could not decode public key");
                Self::verify_signature(public_key_bytes, put_request.signature, vec![put_request.record_value])
            }
            Actions::Get(get_request) => {
                let public_key_bytes = decode(get_request.clone().user_public_key).expect("Could not decode public key");
                Self::verify_signature(public_key_bytes, get_request.signature, vec![get_request.record_key])
            }
            Unknown => {
                Err(DisruptedDataError { message: "Unknown Action, Valid actions are put and get".to_string() })
            }
        }
    }

    pub fn get_record(self) -> Result<Record, DisruptedDataError> {
        match self {
            Actions::Put(put_request) => {
                Ok(put_request.to_record())
            }
            Actions::Get(get_request) => {
                Err(DisruptedDataError{message: "Record not available for Get actions".to_string()})
            }
            Unknown => {
                Err(DisruptedDataError{message: "Unknown action".to_string()})
            }
        }

    }
    pub fn get_record_key_hex(self) -> Result<String, DisruptedDataError> {
        match self {
            Actions::Put(put_request) => {
                Ok(encode(put_request.record_key))
            }
            Actions::Get(get_request) => {
                Ok(encode(get_request.record_key))
            }
            Unknown => {
                Err(DisruptedDataError{message: "Unknown action".to_string()})
            }
        }

    }

    pub fn get_record_key(self) -> Result<RecordKey, DisruptedDataError> {
        match self {
            Actions::Put(put_request) => {
                Ok(put_request.to_record().key)
            }
            Actions::Get(get_request) => {
                Ok(get_request.to_record_key())
            }
            Unknown => {
                Err(DisruptedDataError{message: "Unknown action".to_string()})
            }
        }

    }

    fn verify_signature(user_public_key: Vec<u8>, signature: Vec<u8>, parts: Vec<String>) -> Result<(), DisruptedDataError> {
        let secp256k1_public_key_result = get_secp256k1_public_key(user_public_key);

        match secp256k1_public_key_result {
            Ok(public_key) => {
                if is_identity_verified(signature, public_key, get_message(parts)) {
                    Ok(())
                } else {
                    Err(DisruptedDataError { message: "Could not get public key".to_string() })
                }
            }
            Err(error) => {
                Err(DisruptedDataError { message: "Could not get public key".to_string() })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutRequest {
    pub user_public_key: Vec<u8>,
    pub record_key: String,
    pub record_value: String,
    pub signature: Vec<u8>,
}

impl PutRequest {
    pub fn to_record(mut self) -> Record {
        let mut record_key_bytes: Vec<u8> = self.user_public_key.clone();
        record_key_bytes.append(&mut self.record_key.into_bytes());
        Record::new(RecordKey::new(&encode(record_key_bytes)), self.record_value.as_bytes().to_vec())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetRequest {
    pub user_public_key: Vec<u8>,
    pub record_key: String,
    pub signature: Vec<u8>,
}

impl GetRequest {
    pub fn to_record_key(mut self) -> RecordKey {
        let mut record_key_bytes: Vec<u8> = self.user_public_key.clone();
        record_key_bytes.append(&mut self.record_key.into_bytes());
        RecordKey::new(&encode(record_key_bytes))
    }
}