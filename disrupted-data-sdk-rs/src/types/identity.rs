use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use libp2p::identity::Keypair;
use libp2p::PeerId;
use secp256k1::{Message, Secp256k1};

use crate::hash_message;

pub struct Identity {
    pub key_location: PathBuf,
    pub keypair: Keypair,
}

impl Identity {

    pub fn new(key_location: PathBuf) -> Self {
        Self {
            key_location: key_location.clone(),
            keypair: get_key_pair(&key_location),
        }
    }

    pub fn get_peer_id(&self) -> PeerId {
        PeerId::from(self.keypair.public())
    }

    pub fn sign(secret_key: Vec<u8>, message: String) -> Vec<u8> {
        let secp256k1 = Secp256k1::new();
        let message = Message::from_digest(<[u8; 32]>::try_from(hash_message(&message)).unwrap());
        // let keypair = self.keypair;

        let secp256k1_key_pair = secp256k1::Keypair::from_seckey_slice(&secp256k1, secret_key.as_slice()).unwrap();

        let signature = secp256k1.sign_schnorr(&message, &secp256k1_key_pair);
        signature.serialize().as_slice().to_vec()
    }

}

pub fn get_key_pair(key_location: &PathBuf) -> Keypair {
    let node_key_location_path = key_location.as_path();

    match node_key_location_path.exists() {
        true => {
            get_existing_key(node_key_location_path)
        }
        false => {
            generate_new_key(node_key_location_path)
        }
    }

}

fn get_existing_key(key_location: &Path) -> Keypair {
    let mut file = match File::open(&key_location) {
        Err(why) => panic!("Couldn't open {}: {}", key_location.display(), why.to_string()),
        Ok(file) => file,
    };

    let mut bytes = Vec::new();
    match file.read_to_end(&mut bytes) {
        Err(why) => panic!("Couldn't read {}: {}", key_location.display(), why.to_string()),
        Ok(_) => {}
    };

    Keypair::from_protobuf_encoding(&bytes).expect("Should generate ED25519 key from bytes")
}

fn generate_new_key(node_key_location_path: &Path) -> Keypair {
    let node_key = Keypair::generate_secp256k1();
    let secret_bytes = node_key.to_protobuf_encoding().expect("Should be able to encode into protobuf structure");

    let mut file = match File::create(&node_key_location_path) {
        Err(why) => panic!("Couldn't create {:?} because: {}", node_key_location_path, why.to_string()),
        Ok(file) => file,
    };

    match file.write_all(&secret_bytes) {
        Err(why) => panic!("Couldn't write to {:?} because: {}", node_key_location_path, why.to_string()),
        Ok(_) => {
            println!("Generated news key at {:?}", node_key_location_path);
            node_key
        }
    }
}
