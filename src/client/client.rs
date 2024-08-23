use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn start(&self) {
        println!("Starting client");

        let bound_client_listener = TcpListener::bind("0.0.0.0:6969").await.expect("Could not bind to port 6969");
        println!("Bound client port");

        loop {
            let (mut client_stream, socket_address) = bound_client_listener.accept().await.expect("Error reading client stream");
            println!("Accepted new client.");
            tokio::spawn(async move {
                println!("Handling input from client.");
                let (mut reader, mut writer) = client_stream.split();
                let mut buf = vec![0; 1024];

                loop {
                    match reader.read(&mut buf).await {
                        Ok(0) => {
                            println!("Connection closed");
                            break;
                        }, // connection closed
                        Ok(n) => {
                            let msg = String::from_utf8_lossy(&buf[..n]);
                            println!("Received: {}", msg);
                        }
                        Err(e) => {
                            eprintln!("Failed to read from socket; err = {:?}", e);
                            break;
                        }
                    }
                }
            });
        }
    }
}
