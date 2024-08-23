extern crate core;

use std::fs;

use clap::Parser;
use tokio::sync::mpsc;

use p2p::{FromDisruptedDataSwarmEvent, ToDisruptedDataSwarmEvent};

use crate::p2p::{DisruptedDataSwarm, RequestHandler};
use crate::types::Args;
use crate::types::NodeConfig;

mod p2p;
mod types;
mod client;

#[tokio::main]
async fn main() {
    let (from_swarm_sender, mut from_swarm_receiver) = mpsc::channel::<FromDisruptedDataSwarmEvent>(50);
    let (to_swarm_sender, mut to_swarm_receiver) = mpsc::channel::<ToDisruptedDataSwarmEvent>(50);

    let mut swarm = DisruptedDataSwarm::new(get_node_config(), from_swarm_sender.clone(), to_swarm_receiver);
    let mut request_handler = RequestHandler::new(from_swarm_receiver, to_swarm_sender.clone());

    tokio::spawn(request_handler.process());
    swarm.start().await;
}

fn get_node_config() -> NodeConfig {
    let args = Args::parse();
    match args.key_location {
        None => { NodeConfig::empty() }
        Some(config_file_location) => {
            let config_file_string = fs::read_to_string(config_file_location);
            let config: NodeConfig = toml::from_str(config_file_string.unwrap().as_str()).unwrap();
            config
        }
    }
}
