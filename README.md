# Disrupted Data

There are two main components in this project:

1. Distributed Hash Table (DHT) Node: A node for storing data that can be run by anyone. 
2. SDK: A rust implementation of the SDK that provides an interface to the network.

## DHT Node
For development purposes, the node can be run within using the following commands:

### Start the bootstrap node
>docker compose -f .\compose.yaml up bootstrap

### Start node one
>docker compose -f .\compose.yaml up node-one


## Client

The client can be built using the following cargo command 

``` cargo build -p disrupted-data-client-rs ```

The client uses the SDK to interface with the DHT network:

The client can be started like so -

> disrupted-data-cli.exe --key C:\Nostr\keys\dd-client-2.key

If the key does not exist, a new key pair will be generated at the location, if possible.





