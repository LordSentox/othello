use std::sync::{Arc, Weak, Mutex};
use super::nethandler::NetHandler;
use packets::*;
use std::collections::HashMap;
use bus::BusReader;

/// The master server. It manages the clients, especially when they are not currently
/// in a game.
pub struct Master {
    nethandler: Arc<NetHandler>,
    named_clients: HashMap<ClientId, String>,
    packet_reader: BusReader<(ClientId, Packet)>
}

impl Master {
    /// Start the master server on the specified port. This currently just panics when something
    /// goes wrong, since the program would never run if it is not started up correctly.
    /// If that changes for some reason, this is a TODO.
    pub fn new(port: u16) -> Master {
        // The NetHandler is created and managed by the master server.
        let nethandler = NetHandler::new();
        let packet_reader = nethandler.subscribe_all();

        match nethandler.start_listen(port) {
            Ok(nh) => nh,
            Err(err) => panic!("Could not start listening for clients. {}", err)
        }

        Master {
            nethandler: Arc::new(nethandler),
            named_clients: HashMap::new(),
            packet_reader: packet_reader
        }
    }

    pub fn handle_packets(&mut self) {

    }
}
