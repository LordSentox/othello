use std::sync::{Arc, Mutex};
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
        loop {
            let (client, packet) = match self.packets.lock().unwrap().pop_front() {
                Some(cp) => cp,
                None => break
            };

            match packet {
                Packet::Disconnect => self.handle_disconnect(client),
                Packet::Login(name) => self.handle_login(client, name),
                Packet::Message(to, msg) => self.handle_message(client, to, msg),
                _ => {}
            }
        }
    }

    fn handle_disconnect(&mut self, client: ClientId) {
        if !self.named_clients.contains_key(&client) {
            println!("Unnamed client disconnected. Id [{}]", client);
        }
        else {
            let name = self.named_clients.get(&client).unwrap();
            println!("'{}' disconnected. Id [{}]", name, client);
        }

        self.named_clients.remove(&client);

        // Make sure the clients have the updated client-list.
        self.push_client_list();
    }

    fn handle_login(&mut self, client: ClientId, name: String) {
        // If the name is already in use, the login fails.
        for taken in self.named_clients.values() {
            if &name == taken {
                self.nethandler.send(client, &Packet::LoginDeny("Name already in use.".to_string()));
				return;
            }
        }

        // The name is not taken yet. Add the client to the named_clients and return the message of
        // success to the client.
        if self.nethandler.send(client, &Packet::LoginAccept) {
            self.named_clients.insert(client, name.clone());

            // Make sure the clients have the updated list.
            self.push_client_list();

			println!("Client [{}] logged in as '{}'", client, name);
        }
        else {
            println!("Client [{}] tried to login as [{}] (available), but the accept message could not be sent.", client, name);
        }
    }

    /// Whenever a client logs in, changes name or is disconnected, this can be called to update
    /// the client list on all clients, letting them know the current state. This way it is
    /// assured the client always has the correct information without always having to ask first.
    pub fn push_client_list(&self) {
        let clients_vec: Vec<(ClientId, String)> = self.named_clients.clone().into_iter().collect();

        self.nethandler.broadcast(&Packet::ClientList(clients_vec));
    }

    fn handle_message(&self, from: ClientId, to: ClientId, message: String) {
        // At the moment, the message will simply be passed on unchecked. Later there will probably
        // be things like SPAM-Filter etc.
        self.nethandler.send(to, &Packet::Message(from, message));
    }

    pub fn get_login_name(&self, client: ClientId) -> Option<String> {
        match self.named_clients.get(&client) {
            Some(ref name) => Some(name.to_string()), // XXX: Why?!? to_string() ? On a string?
            None => None
        }
    }

    pub fn get_id(&self, login_name: &str) -> Option<ClientId> {
        for (id, name) in &self.named_clients {
            if name.as_str() == login_name {
                return Some(*id)
            }
        }

        None
    }
}
