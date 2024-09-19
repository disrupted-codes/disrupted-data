use secp256k1::{Message, PublicKey, Secp256k1};
use secp256k1::schnorr::Signature;
use disrupted_data_sdk_rs::DisruptedDataError;

pub(crate) mod request;
pub(crate) mod state;

pub(crate) fn is_identity_verified(signature_vec: Vec<u8>, public_key: PublicKey, message: String) -> bool {
	let secp256k1 = Secp256k1::new();

	let message = Message::from_digest(<[u8; 32]>::try_from(disrupted_data_sdk_rs::hash_message(&message)).unwrap());
	let (x_only_public_key, parity) = public_key.x_only_public_key();
	let signature = Signature::from_slice(signature_vec.as_slice()).unwrap();
	secp256k1.verify_schnorr(&signature, &message, &x_only_public_key).is_ok()
}

fn get_secp256k1_public_key(public_key: Vec<u8>) -> Result<PublicKey, DisruptedDataError> {
	PublicKey::from_slice(public_key.as_slice()).map_err(|e| { DisruptedDataError { message: "Invalid Key".to_string() } })
}

fn get_message(parts: Vec<String>) -> String {
	parts.into_iter()
		.map(|s| s.trim().to_string())
		.collect()
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
