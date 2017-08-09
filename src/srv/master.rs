use std::sync::{Arc, Weak, Mutex};
use super::nethandler::NetHandler;
use packets::*;
use std::collections::{HashMap, VecDeque};

/// The master server. It manages the clients, especially when they are not currently
/// in a game.
pub struct Master {
    nethandler: Arc<NetHandler>,
    named_clients: HashMap<ClientId, String>,
    packets: Arc<Mutex<VecDeque<(ClientId, Packet)>>>
}

impl Master {
    /// Start the master server on the specified port. This currently just panics when something
    /// goes wrong, since the program would never run if it is not started up correctly.
    /// If that changes for some reason, this is a TODO.
    pub fn new(nethandler: Arc<NetHandler>) -> Master {
        // Create the packets VecDeque and subscribe to the server.
        let packets = Arc::new(Mutex::new(VecDeque::new()));
        nethandler.subscribe(Arc::downgrade(&packets));

        Master {
            nethandler: nethandler,
            named_clients: HashMap::new(),
            packets: packets
        }
    }

    pub fn handle_packets(&mut self) {
    }
}
