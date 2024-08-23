use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use toml::{Table, Value};
use toml::map::Map;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeConfig {
    pub ip_address: Option<String>,
    pub port: Option<String>,
    pub node_key_location: Option<PathBuf>,
    pub bootstrap_nodes: Option<Table>,
    pub log_file: Option<String>,
}

impl NodeConfig {
    pub fn empty() -> Self {
        Self {
            ip_address: None,
            port: None,
            node_key_location: None,
            bootstrap_nodes: None,
            log_file: None,
        }
    }
    pub fn ip_address(&self) -> String {
        if self.ip_address.is_some() {
            let ip_address_clone = self.ip_address.clone().unwrap();
            ip_address_clone
        } else {
            env::var("IP_ADDRESS").unwrap_or_else(|e| { "127.0.0.1".to_string() })
        }
    }
    pub fn port(&self) -> String {
        if self.port.is_some() {
            let port_clone = self.port.clone().unwrap();
            port_clone
        } else {
            env::var("PORT").unwrap_or_else(|e| { "6969".to_string() })
        }
    }
    pub fn node_key_location(self) -> PathBuf {
        self.node_key_location.unwrap_or_else(|| {
            let env_node_key_location = env::var("NODE_KEY_LOCATION").unwrap();
            Path::new(&env_node_key_location).to_path_buf()
        })
    }
    pub fn bootstrap_nodes(&self) -> Table {
        if self.bootstrap_nodes.is_some() {
            let bootstrap_nodes_clone = self.bootstrap_nodes.clone();
            bootstrap_nodes_clone.unwrap()
        } else {
            let mut boostrap_nodes_map: Table = Map::new();
            boostrap_nodes_map.insert("1AXGCQH1hrhZ99MTDDJbMYZD6AYfsfACVDDVZRPxm5KWnj".to_string(), Value::String("189.90.0.2".to_string()));
            let environment_bootstrap_nodes = env::var("BOOTSTRAP_NODES");
            match environment_bootstrap_nodes {
                Ok(env_bootstrap_nodes) => {
                    let env_bootstrap_nodes = parse_string_to_table(env_bootstrap_nodes);
                    env_bootstrap_nodes
                }
                Err(err) => {
                    boostrap_nodes_map
                }
            }
        }
    }
    pub fn log_file(&self) -> String {
        if self.log_file.is_some() {
            let log_file_clone = self.log_file.clone().unwrap();
            log_file_clone
        } else {
            env::var("LOG_FILE").unwrap_or_else(|e| { "C:\\Nostr\\disrupted-data\\disrupted-data.log".to_string() })
        }
    }
}

fn parse_string_to_table(env_bootstrap_nodes: String) -> Table {
    let mut boostrap_nodes_map: Table = Map::new();

    let cleaned_str = env_bootstrap_nodes.trim_matches(|c| c == '{' || c == '}');
    let parts: Vec<&str> = cleaned_str.split("=").collect();

    if parts.len() == 2 {
        println!("part[0]: {}", parts[0]);
        println!("part[1]: {}", parts[1]);
        let key = parts[0].trim().trim_matches('"').to_string();
        let value = parts[1].trim().trim_matches('"').to_string();
        boostrap_nodes_map.insert(key, Value::String(value));
    };
    boostrap_nodes_map
}