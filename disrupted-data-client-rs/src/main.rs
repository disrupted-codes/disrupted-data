use clap::Parser;
use rustyline::DefaultEditor;

use disrupted_data_sdk_rs::{Actions, Client, DisruptedDataError};
use disrupted_data_sdk_rs::Identity;

use crate::types::Args;

mod types;

#[tokio::main]
async fn main() {
    let mut arg = Args::parse();
    let identity = Identity::new(arg.key);
    let ip = arg.ip.get_or_insert("127.0.0.1".to_string());
    let new_client_result = Client::new(&identity.keypair, ip.clone(), "6969".to_string());
    // let new_client_result = Client::new(&identity.keypair, "127.0.0.1".to_string(), "6969".to_string());

    match new_client_result {
        Ok(client) => {
            prompt(client, &identity).await;
        }
        Err(error) => {
            println!("Aborting. Could not connect to the node");
        }
    }
}

async fn prompt(mut client: Client, identity: &Identity) {
    let mut line = DefaultEditor::new().unwrap();

    loop {
        let user_input = line.readline("disrupted-data >> ").unwrap();

        let user_action: Actions = (user_input, identity).into();
        if let Actions::Unknown = user_action {
            println!("Usage:");
            println!("put <<Data key>> <<Data value>>");
            println!("get <<Data key>>");
            continue;
        }

        let action_result = client.process_action(user_action).await;
        match action_result {
            Ok(action_result) => {
                println!("Response: {:?}", action_result.get_message())
            }
            Err(err) => {
                println!("Error executing action: {}", err)
            }
        }

    }
}
