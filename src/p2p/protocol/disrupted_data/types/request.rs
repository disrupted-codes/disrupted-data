use crate::p2p::protocol::disrupted_data::request::Request::{GET, PUT};
use crate::p2p::protocol::disrupted_data::types::state::{GetRequestState, PutRequestState};
use crate::p2p::protocol::disrupted_data::types::verify_signature;
use disrupted_data_sdk_rs::{Actions, DisruptedDataError};
use hex::{decode, encode};
use libp2p::kad::{QueryId, Record, RecordKey};
use libp2p::request_response::InboundRequestId;

#[derive(Debug, Clone)]
pub(crate) enum Request {
	PUT(PutRequestState, RequestData),
	GET(GetRequestState, RequestData),
}

impl TryFrom<(InboundRequestId, Actions)> for Request {
	type Error = DisruptedDataError;


	fn try_from(value: (InboundRequestId, Actions)) -> Result<Self, Self::Error> {
		match value.1 {
			Actions::Put(put_request) => {
				let request_data = RequestData {
					// peer_id: value.0,
					inbound_request_id: value.0,
					user_public_key: put_request.user_public_key,
					record_key: put_request.record_key,
					record_value: Some(put_request.record_value),
					signature: put_request.signature,
					user: None,
				};

				Ok(
					PUT(PutRequestState::Verify, request_data)
				)
			}
			Actions::Get(get_request) => {
				let request_data = RequestData {
					// peer_id: value.0,
					inbound_request_id: value.0,
					user_public_key: get_request.user_public_key,
					record_key: get_request.record_key,
					record_value: None,
					signature: get_request.signature,
					user: None,
				};
				Ok(
					GET(GetRequestState::Verify, request_data)
				)
			}
			Actions::Unknown => { Err(DisruptedDataError { message: format!("Unknown action: {:?}", value.1) }) }
		}
	}
}

impl Request {
	pub(crate) fn verify_request(&self) -> VerifyRequestResult {
		match self {
			PUT(_, data) => {
				let public_key_bytes = decode(data.clone().user_public_key).expect("Could not decode public key");
				if let Ok(()) = verify_signature(public_key_bytes, data.clone().signature, vec![data.clone().record_value.unwrap()])
				{ VerifyRequestResult::Success } else { VerifyRequestResult::Failed(DisruptedDataError { message: "Unexpected identity signature".to_string() }) }
			}
			GET(_, data) => {
				let public_key_bytes = decode(data.clone().user_public_key).expect("Could not decode public key");
				if let Ok(()) = verify_signature(public_key_bytes, data.clone().signature, vec![data.record_key.clone()])
				{ VerifyRequestResult::Success } else { VerifyRequestResult::Failed(DisruptedDataError { message: "Unexpected identity signature".to_string() }) }
			}
		}
	}

	pub(crate) fn get_data(&self) -> RequestData {
		match self {
			PUT(_, data) | GET(_, data) => {
				data.clone()
			}
		}
	}

	pub(crate) fn get_user_public_key(self) -> Vec<u8> {
		match self {
			PUT(_, data) => {
				data.user_public_key
			}
			GET(_, data) => {
				data.user_public_key
			}
		}
	}
}


#[derive(Debug, Clone)]
pub(crate) enum GetInboundRequestState {
	New,
	FindUser,
	FindRecord,
	SendResponse,
}


#[derive(Debug, Clone)]
pub(crate) enum FindResult {
	Found(QueryId, Record),
	NotFound,
}


#[derive(Debug, Clone)]
pub(crate) enum PutResult {
	NotExecuted,
	Success,
	Failed(DisruptedDataError),
}

#[derive(Debug, Clone)]
pub(crate) enum VerifyRequestResult {
	Success,
	Failed(DisruptedDataError),
}


#[derive(Debug, Clone)]
pub(crate) struct RequestData {
	// peer_id: PeerId,
	inbound_request_id: InboundRequestId,
	user_public_key: Vec<u8>,
	pub(crate) record_key: String,
	pub(crate) record_value: Option<String>,
	signature: Vec<u8>,
	user: Option<Record>,
}

impl RequestData {
	pub(crate) fn get_record(&self) -> Record {
		let mut record_key_bytes: Vec<u8> = self.user_public_key.clone();
		record_key_bytes.append(&mut self.record_key.clone().into_bytes());

		//Record can only be created for Put record which should have the record_value
		Record::new(RecordKey::new(&encode(record_key_bytes)), self.record_value.clone().unwrap().as_bytes().to_vec())
	}

	pub(crate) fn get_user_record_key(&self) -> RecordKey {
		let mut record_key_bytes: Vec<u8> = self.user_public_key.clone();
		// record_key_bytes.append(&mut self.record_key.clone().into_bytes());
		RecordKey::new(&record_key_bytes)
		// RecordKey::new(&encode(record_key_bytes))
	}

	pub(crate) fn get_data_record_key(self) -> RecordKey {
		let mut record_key_bytes: Vec<u8> = self.user_public_key.clone();
		record_key_bytes.append(&mut self.record_key.into_bytes());
		RecordKey::new(&encode(record_key_bytes))
	}

	pub(crate) fn update_request_with_user(mut self, user: Option<Record>) -> Self {
		match user {
			None => {
				let mut user_data_keys: Vec<u8> = Vec::new();
				let new_key = self.record_key.as_bytes().to_vec();
				let length = new_key.len() as u32;
				user_data_keys.extend_from_slice(&length.to_le_bytes());
				user_data_keys.extend_from_slice(&new_key);
				let user_record_key = RecordKey::new(&encode(self.user_public_key.clone()));
				let new_user_record = Record::new(user_record_key, user_data_keys);

				Self {
					// peer_id: self.peer_id,
					inbound_request_id: self.inbound_request_id,
					user_public_key: self.user_public_key,
					record_key: self.record_key,
					record_value: self.record_value,
					signature: self.signature,
					user: Some(new_user_record),
				}
			}
			Some(user_record) => {
				let mut user_data_keys = user_record.clone().value;
				let new_key = self.record_key.as_bytes().to_vec();
				let length = new_key.len() as u32;
				user_data_keys.extend_from_slice(&length.to_le_bytes());
				user_data_keys.extend_from_slice(&new_key);
				let updated_user_record = Record::new(user_record.key, user_data_keys);
				Self {
					// peer_id: self.peer_id,
					inbound_request_id: self.inbound_request_id,
					user_public_key: self.user_public_key,
					record_key: self.record_key,
					record_value: self.record_value,
					signature: self.signature,
					user: Some(updated_user_record),
				}
			}
		}
	}

	pub(crate) fn add_data_key_to_user(mut self) -> Self {
		let mut user_data_keys = self.user.clone().unwrap().value;
		let new_key = self.record_key.as_bytes().to_vec();
		let length = new_key.len() as u32;
		user_data_keys.extend_from_slice(&length.to_le_bytes());
		user_data_keys.extend_from_slice(&new_key);
		let updated_user_record = Record::new(self.user.unwrap().key, user_data_keys);
		Self {
			// peer_id: self.peer_id,
			inbound_request_id: self.inbound_request_id,
			user_public_key: self.user_public_key,
			record_key: self.record_key,
			record_value: self.record_value,
			signature: self.signature,
			user: Some(updated_user_record),
		}
	}
}
