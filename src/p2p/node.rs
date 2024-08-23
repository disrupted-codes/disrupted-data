use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use libp2p::identity::Keypair;
use libp2p::PeerId;
use tokio::sync::oneshot;
use uuid::Uuid;

use disrupted_data_sdk_rs::Identity;

use crate::types::config::NodeConfig;

#[derive(Clone)]
pub struct Node {
    pub ip_address: String,
    pub port: String,
    pub log_file: String,
    pub key: Keypair,
    pub peer_id: PeerId,
    clients: Arc<Mutex<HashMap<Uuid, oneshot::Sender<Vec<u8>>>>>,
    get_requests: Arc<Mutex<HashMap<String, Uuid>>>,

}

impl Node {
    pub fn new(config: NodeConfig) -> Self {
        let identity = Identity::new(config.clone().node_key_location());

        Self {
            ip_address: config.ip_address(),
            port: config.port(),
            log_file: config.log_file(),
            key: identity.keypair.clone(),
            peer_id: identity.get_peer_id(),
            clients: Arc::new(Mutex::new(HashMap::<Uuid, oneshot::Sender<Vec<u8>>>::new())),
            get_requests: Arc::new(Mutex::new(HashMap::new())),

        }
    }
}

