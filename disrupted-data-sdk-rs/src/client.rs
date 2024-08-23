use futures::StreamExt;
use libp2p::{PeerId, request_response, Swarm};
use libp2p::identity::Keypair;
use libp2p::request_response::Message;
use libp2p::swarm::SwarmEvent;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{Actions, behaviour};
use crate::behaviour::UserNodeBehaviour;
use crate::connection::Connection;
use crate::types::error::DisruptedDataError;

pub struct Client {
    user_command_sender: Sender<Actions>,
}


impl Client {
    pub fn new(user_keypair: &Keypair, node_ip: String, node_port: String) -> Result<Self, DisruptedDataError> {
        let (tx, rx) = channel(400);

        let swarm = Connection::connect_swarm(user_keypair, node_ip.clone(), node_port.clone())?;
        tokio::spawn(async move { Self::listen_for_user_input(swarm, rx).await });

        Ok(Self {
            user_command_sender: tx
        })
    }

    pub async fn process_action(&mut self, put_action: Actions) -> Result<(), DisruptedDataError> {
        let send_result = self.user_command_sender.send(put_action).await;

        match send_result {
            Ok(_) => {
                println!("Sent command");
                Ok(())
            }
            Err(error) => {
                println!("Error sending command: {}", error);
                Err(DisruptedDataError { message: format!("Error sending command: {}", error) })
            }
        }
    }

    async fn listen_for_user_input(mut swarm: Swarm<UserNodeBehaviour>, mut user_command_receiver: Receiver<Actions>) {
        let mut connected_peer_id: Option<PeerId> = None;

        loop {
            select! {
                Some(action) = user_command_receiver.recv() => {
                    eprintln!("user action: {:?} connected peer: {:?}", action, connected_peer_id);
                     if connected_peer_id.is_some() {
                         eprintln!("found connected peer...sending request");
                        Some(swarm.behaviour_mut().request_response.send_request(&connected_peer_id.unwrap(), action.clone()));
                     } else {
                         eprintln!("could not find connected peer");

                     }

                },

                swarm_event = swarm.select_next_some() => {
                    match swarm_event {
                        SwarmEvent::ConnectionEstablished {peer_id, ..} => {
                            connected_peer_id = Some(peer_id);
                        },
                        SwarmEvent::Behaviour(behaviour::Event::RequestResponse(request_response::Event::Message {message, .. } ) ) => {
                            match message {
                                Message::Request { request_id, request, .. } => {
                                    eprintln!("Request received: {:?}", request);
                                }
                                Message::Response { request_id, response } => {
                                    eprintln!("Response received: {:?}", response);
                                }
                            }
                        },
                        _ => {
                        }
                    }
                }
            }
        }
    }
}
