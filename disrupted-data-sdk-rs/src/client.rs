use std::collections::HashMap;
use futures::StreamExt;
use libp2p::{PeerId, request_response, Swarm};
use libp2p::identity::Keypair;
use libp2p::request_response::{Message, OutboundRequestId};
use libp2p::swarm::SwarmEvent;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

use crate::{ActionResult, Actions, behaviour};
use crate::behaviour::UserNodeBehaviour;
use crate::connection::Connection;
use crate::types::error::DisruptedDataError;

pub struct Client {
    user_command_sender: Sender<(Actions, oneshot::Sender<ActionResult>)>
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

    pub async fn process_action(&mut self, put_action: Actions) -> Result<ActionResult, DisruptedDataError> {
        let (user_command_response_sender, user_command_response_receiver) = oneshot::channel::<ActionResult>();
        let send_result = self.user_command_sender.send((put_action, user_command_response_sender)).await;

        match send_result {
            Ok(_) => {
                Ok(user_command_response_receiver.await.unwrap())
                // Ok(())
            }
            Err(error) => {
                // println!("Error sending command: {}", error);
                Err(DisruptedDataError { message: format!("Error sending command: {}", error) })
            }
        }
    }

    async fn listen_for_user_input(mut swarm: Swarm<UserNodeBehaviour>, mut user_command_receiver: Receiver<(Actions, oneshot::Sender<ActionResult>)>) {
        let mut connected_peer_id: Option<PeerId> = None;
        let mut request_id_response_channel_map = HashMap::<OutboundRequestId, oneshot::Sender<ActionResult>>::new();

        loop {
            select! {
                Some((action, user_command_response_sender)) = user_command_receiver.recv() => {
                    // println!("user action: {:?} connected peer: {:?}", action, connected_peer_id);
                     if connected_peer_id.is_some() {
                         // println!("found connected peer...sending request");
                        let request_id = swarm.behaviour_mut().request_response.send_request(&connected_peer_id.unwrap(), action.clone());
                        request_id_response_channel_map.insert(request_id, user_command_response_sender);

                    } else {
                         // println!("could not find connected peer");

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
                                    // println!("Request received: {:?}", request);
                                }
                                Message::Response { request_id, response } => {
                                    let response_channel_option = request_id_response_channel_map.remove(&request_id);
                                    match response_channel_option {
                                        Some(response_channel) => {
                                            response_channel.send(response).unwrap()
                                        },
                                        None => {
                                            eprintln!("Could not find command response channel");
                                        }
                                    }
                                    // println!("Response received: {:?}", response);
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
