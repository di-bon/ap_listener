# Listener
This repo contains the code for the Listener for the Advanced Programming course help at the University of Trento in the academic year 2024/2025.

## Description
The Listener will receive all the incoming ```Packet```s from the connected drones.
For every received ```Packet```, it will communicate with the Transmitter 
in order to send an appropriate response, which can either be an ```Ack```, 
a ```Nack```, a forwarded ```FloodRequest``` or a ```FloodResponse```.

The Listener stores the received message fragments, and will use the 
Assembler when all the fragments for a given session_id are received.
Then, the Message is forwarded to Logic, which can either be a Client
or a Server, to process the high-level message appropriately.

## Usage
To use the Listener, add
```toml
ap_listener = { git = "https://github.com/di-bon/ap_listener.git" }
```
to your Cargo.toml file.

Then, import it in your project files using
```rust
use ap_listener::Listener;
```

To create a new Listener, use the constructor ```Listener::new```.

To make the Listener work and process Packets, call ```Listener::run()```

## Panics
See the documentation for each function.
