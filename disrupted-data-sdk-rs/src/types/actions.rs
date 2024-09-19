use std::str::FromStr;

use hex::encode;
use libp2p::kad::{Record, RecordKey};
use serde::{Deserialize, Serialize};

use crate::types::actions::Actions::Unknown;
use crate::{DisruptedDataError, Identity};

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

impl ActionResult {
	pub fn get_message(&self) -> String {
		match self {
			ActionResult::Success(message) | ActionResult::Failure(message) => {
				message.clone()
			}
		}
	}
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
	pub fn get_record(self) -> Result<Record, DisruptedDataError> {
		match self {
			Actions::Put(put_request) => {
				Ok(put_request.to_record())
			}
			Actions::Get(get_request) => {
				Err(DisruptedDataError { message: "Record not available for Get actions".to_string() })
			}
			Unknown => {
				Err(DisruptedDataError { message: "Unknown action".to_string() })
			}
		}
	}

	pub fn get_user_public_key(self) -> Result<Vec<u8>, DisruptedDataError> {
		match self {
			Actions::Put(put_request) => {
				Ok(put_request.user_public_key)
			}
			Actions::Get(get_request) => {
				Ok(get_request.user_public_key)
			}
			Unknown => {
				Err(DisruptedDataError { message: "Unknown action".to_string() })
			}
		}
	}

	pub fn get_record_key_hex(self) -> Result<String, DisruptedDataError> {
		match self {
			Actions::Put(put_request) => {
				let mut raw_record_key = put_request.user_public_key.clone();
				raw_record_key.append(&mut put_request.record_key.into_bytes());
				Ok(encode(raw_record_key))
			}
			Actions::Get(get_request) => {
				let mut raw_record_key = get_request.user_public_key.clone();
				raw_record_key.append(&mut get_request.record_key.into_bytes());

				Ok(encode(raw_record_key))
			}
			Unknown => {
				Err(DisruptedDataError { message: "Unknown action".to_string() })
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
				Err(DisruptedDataError { message: "Unknown action".to_string() })
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