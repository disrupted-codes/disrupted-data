pub mod error;
pub mod actions;
pub mod identity;

use std::io::Write;
use sha2::Digest;
use sha2::digest::Update;
pub use identity::Identity;
pub use error::DisruptedDataError;
pub use actions::PutRequest;
pub use actions::GetRequest;
use secp256k1::{Message, PublicKey, Secp256k1};
use secp256k1::schnorr::Signature;


pub fn is_identity_verified(signature_vec: Vec<u8>, public_key: PublicKey, message: String) -> bool {
    let secp256k1 = Secp256k1::new();

    let message = Message::from_digest(<[u8; 32]>::try_from(crate::hash_message(&message)).unwrap());
    let (x_only_public_key, parity) = public_key.x_only_public_key();
    let signature = Signature::from_slice(signature_vec.as_slice()).unwrap();
    secp256k1.verify_schnorr(&signature, &message, &x_only_public_key).is_ok()
}

pub fn get_secp256k1_public_key(public_key: Vec<u8>) -> Result<PublicKey, DisruptedDataError> {
    PublicKey::from_slice(public_key.as_slice()).map_err(|e| { DisruptedDataError { message: "Invalid Key".to_string() } })
}