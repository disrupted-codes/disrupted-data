use std::time::Duration;

use futures::StreamExt;
use libp2p::{Multiaddr, noise, PeerId, Swarm, swarm, tcp, Transport, yamux};
use libp2p::core::upgrade::Version;
use libp2p::identity::Keypair;
use libp2p::swarm::DialError;

use crate::behaviour::UserNodeBehaviour;
use crate::types::error::DisruptedDataError;

pub struct Connection {
    swarm: Swarm<UserNodeBehaviour>,
}

impl Connection {
    pub fn connect_swarm(user_keypair: &Keypair, node_ip: String, node_port: String) -> Result<Swarm<UserNodeBehaviour>, DisruptedDataError> {
        let behaviour = UserNodeBehaviour::new();
        let peer_id = PeerId::random();

        let transport = tcp::tokio::Transport::default().upgrade(Version::V1).authenticate(noise::Config::new(&user_keypair).expect("Could not initialise noise")).multiplex(yamux::Config::default()).boxed();
        let swarm_config = swarm::Config::with_tokio_executor().with_idle_connection_timeout(Duration::from_secs(60));
        let mut swarm = Swarm::new(transport, behaviour, peer_id, swarm_config);

        let address_result: libp2p::multiaddr::Result<Multiaddr> = format!("/ip4/{}/tcp/{}", node_ip, node_port).parse();

        match address_result {
            Ok(address) => {
                match swarm.dial(address).map_err(|dial_error: DialError| { DisruptedDataError { message: format!("Could not dial the node{}", dial_error.to_string()) } }) {
                    Ok(_) => { println!("Successfully dialed node") }
                    Err(error) => { return Err(error) }
                }

                let connection = Self {
                    swarm
                };
                Ok(connection.swarm)
            }
            Err(error) => {
                return Err(DisruptedDataError { message: format!("Error parsing multiaddress: {}", error) })
            }
        }
    }
}