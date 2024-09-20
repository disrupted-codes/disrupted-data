# Disrupted Data

## Trying the prototype browser client

1. Clone the repository
``` git clone git@github.com:disrupted-codes/disrupted-data.git ```

2. Build client
```cargo build -p disrupted-data-client-rs```

3. Connect to the node.
```.\disrupted-data-cli --key /keys/dd-client.key --ip 170.64.140.33 ```
--key -> Your Secp256k1 keypair. If the key file does not exist, it will be created.
--ip -> The DHT node to connect to. 170.64.140.33 can be used.

4. Put data
```put <<Data key>> <<Data value>>```
Eg. ```put hello world```

5. Get data by key
```get <<Data key>>```
Eg. ```get hello```



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

## TODO

- [x] Add Disrupted data behaviour (In progress).
- 🟠 Add logging and tracing.
- [ ] Add merkle tree to support grouping of user data.
- [ ] disrupted-data Nostr client
- [ ] Data fragmentation and joining to support.
- [ ] disrupted-data Git client 




