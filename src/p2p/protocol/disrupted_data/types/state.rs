use crate::p2p::protocol::disrupted_data::request::{FindResult, RequestData, VerifyRequestResult};
use crate::p2p::protocol::disrupted_data::types::state::GetRequestState::{CouldNotGetData, DataNotAssociatedWithUser, FindData, WaitingData};
use crate::p2p::protocol::disrupted_data::types::state::PutRequestState::{CreateDataRecord, FindUser, SendResponse, WaitingDataCreate, WaitingPut, WaitingUserCreate};
use crate::p2p::User;
use disrupted_data_sdk_rs::ActionResult;
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::{QueryId, Quorum, RecordKey};
use std::collections::HashSet;

pub(crate) trait RequestState {}

#[derive(Debug, Clone)]
pub(crate) enum PutRequestState {
	Verify,
	FindUser(QueryId),
	CreateUserRecord(FindResult),
	WaitingUserCreate(QueryId),
	CreateDataRecord(FindResult),
	WaitingDataCreate(QueryId, FindResult),
	WaitingPut(HashSet<QueryId>),
	SendResponse(ActionResult),

}

impl PutRequestState {
	pub(crate) fn verify(&self, kad: &mut kad::Behaviour<MemoryStore>, verification_result: VerifyRequestResult, user_key: RecordKey) -> Self {
		if let PutRequestState::Verify = self {
			println!("Verification result: {:?}", verification_result);
			match verification_result {
				VerifyRequestResult::Success => {
					println!("user public key: {:?}", user_key);
					let get_user_query_id = kad.get_record(user_key);
					println!("get_user_query_id: {:?}", get_user_query_id);
					FindUser(get_user_query_id)
				}
				VerifyRequestResult::Failed(_) => {
					SendResponse(ActionResult::Failure("Invalid request".to_string()))
				}
			}
		} else {
			SendResponse(ActionResult::Failure("Invalid state".to_string()))
		}
	}

	pub(crate) fn find_user_result(&self, find_user_result: FindResult) -> Self {
		CreateDataRecord(find_user_result)
	}

	pub(crate) fn create_data_record(&self, kad: &mut kad::Behaviour<MemoryStore>, data: RequestData) -> Self {
		if let PutRequestState::CreateDataRecord(find_user_result) = self {
			let data_put_result = kad.put_record(data.get_record(), Quorum::One);
			match data_put_result {
				Ok(data_put_query_id) => {
					println!("data_put_query_id in create_data_record put request state: {:?}", data_put_query_id);
					WaitingDataCreate(data_put_query_id, find_user_result.clone())
				}
				Err(error) => {
					SendResponse(ActionResult::Failure(format!("Error while putting data: {:?}", error)))
				}
			}
		} else {
			SendResponse(ActionResult::Failure("Invalid state".to_string()))
		}
	}
	pub(crate) fn create_or_update_user(&self, kad: &mut kad::Behaviour<MemoryStore>, data: RequestData) -> Self {
		if let PutRequestState::WaitingDataCreate(data_create_query_id, find_user_result) = self {
			let user_record = match find_user_result {
				FindResult::Found(query_id, record) => {
					let mut existing_user: User = record.clone().try_into().unwrap();
					existing_user.add_data_record_keys(data.record_key.clone());
					println!("existing_user with new data record key: {:?}", existing_user);
					existing_user.try_into().unwrap()
					// println!("updated_user_record: {:?}", updated_user_record);
				}
				FindResult::NotFound => {
					let mut new_user = User::new(&data.get_user_record_key());
					new_user.add_data_record_keys(data.record_key.clone());
					println!("new user with new data record key: {:?}", new_user);
					new_user.try_into().unwrap()
					// println!("updated_user_record (New user): {:?}", updated_user_record);
				}
			};
			let user_put_result = kad.put_record(user_record, Quorum::One);
			match user_put_result {
				Ok(user_put_query_id) => {
					WaitingUserCreate(user_put_query_id)
				}
				Err(error) => {
					println!("Error while putting user: {:?}", error);
					SendResponse(ActionResult::Failure("Error while putting user".to_string()))
				}
			}
		} else {
			SendResponse(ActionResult::Failure("Invalid state".to_string()))
		}
	}

	pub(crate) fn update_waiting_put_request_list(&self, completed_query_id: QueryId) -> Self {
		if let WaitingPut(query_id_list) = self {
			let mut new_query_id_set = query_id_list.clone();
			let removal_status = new_query_id_set.remove(&completed_query_id);
			if removal_status {
				println!("Value removed from set");
			} else {
				println!("Value not removed from set");
			}
			if new_query_id_set.is_empty() {
				SendResponse(ActionResult::Success("Completed".to_string()))
			} else {
				println!("query_id_list count: {:?}", query_id_list.len());
				WaitingPut(new_query_id_set)
			}
		} else {
			SendResponse(ActionResult::Failure("Invalid state".to_string()))
		}
	}
}

#[derive(Debug, Clone)]
pub(crate) enum GetRequestState {
	Verify,
	FindUser(QueryId),
	UserNotFound,
	DataNotAssociatedWithUser,
	FindData(User),
	WaitingData(QueryId),
	CouldNotGetData,
	SendResponse(ActionResult),
	Invalid,
}


impl GetRequestState {
	pub(crate) fn verify(&self, kad: &mut kad::Behaviour<MemoryStore>, verification_result: VerifyRequestResult, user_key: RecordKey) -> Self {
		if let GetRequestState::Verify = self {
			println!("Verification result: {:?}", verification_result);
			match verification_result {
				VerifyRequestResult::Success => {
					println!("user public key: {:?}", user_key);
					let get_user_query_id = kad.get_record(user_key);
					println!("get_user_query_id: {:?}", get_user_query_id);
					GetRequestState::FindUser(get_user_query_id)
				}
				VerifyRequestResult::Failed(_) => {
					GetRequestState::SendResponse(ActionResult::Failure("Invalid request".to_string()))
				}
			}
		} else {
			GetRequestState::SendResponse(ActionResult::Failure("Invalid state".to_string()))
		}
	}

	pub(crate) fn find_user_result(&self, find_user_result: FindResult) -> Self {
		match find_user_result {
			FindResult::Found(_, record) => { FindData(record.try_into().unwrap()) }
			FindResult::NotFound => { DataNotAssociatedWithUser }
		}
	}
	pub(crate) fn find_data_record(&self, kad: &mut kad::Behaviour<MemoryStore>, data: RequestData) -> Self {
		let get_data_query_id = kad.get_record(data.get_data_record_key());
		// println!("get_data_query_id in GetRequestState find_data_record : {:?}", get_data_query_id);
		WaitingData(get_data_query_id)
	}

	pub(crate) fn find_data_result(&self, find_user_result: FindResult) -> Self {
		match find_user_result {
			FindResult::Found(_, record) => {
				println!("Found: {:?}", record);
				GetRequestState::SendResponse(ActionResult::Success(String::from_utf8(record.value).unwrap()))
			}
			FindResult::NotFound => { CouldNotGetData }
		}
	}
}