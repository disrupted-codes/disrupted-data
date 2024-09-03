use clap::Parser;
use rustyline::DefaultEditor;

use disrupted_data_sdk_rs::{Actions, Client, DisruptedDataError};
use disrupted_data_sdk_rs::Identity;

use crate::types::Args;

mod types;

#[tokio::main]
async fn main() {
    let arg = Args::parse();
    let identity = Identity::new(arg.key);
    let new_client_result = Client::new(&identity.keypair, "127.0.0.1".to_string(), "6969".to_string());

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

        let action_result = client.process_action(user_action).await;
        match action_result {
            Ok(action_result) => {
                println!("Result: {:?}", action_result)
            }
            Err(err) => {
                println!("Error executing action: {}", err)
            }
        }

    }
}
