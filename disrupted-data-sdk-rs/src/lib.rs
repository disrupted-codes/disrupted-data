use sha2::{Digest, Sha256};

pub use client::Client;
pub use types::actions::ActionResult;
pub use types::actions::Actions;
pub use types::DisruptedDataError;
pub use types::GetRequest;
pub use types::Identity;
pub use types::PutRequest;

mod client;
mod types;
mod connection;
mod behaviour;

pub fn hash_message(message: &String) -> [u8; 32] {
    let mut sha256_hasher = Sha256::new();

    sha256_hasher.update(message);
    let hash: [u8; 32] = sha256_hasher.finalize().as_slice().try_into().unwrap();
    hash
}

pub fn hash_message_u8(message: Vec<u8>) -> [u8; 32] {
    let mut sha256_hasher = Sha256::new();

    sha256_hasher.update(message);
    let hash: [u8; 32] = sha256_hasher.finalize().as_slice().try_into().unwrap();
    hash
}

pub fn get_message(parts: Vec<String>) -> String {
    parts.into_iter()
        .map(|s| s.trim().to_string())
        .collect()
}