use std::sync::{Arc, Mutex, RwLock, Weak};
use std::net::TcpStream;
use super::{ClientId, NetHandler, Remote};
use packets::Packet;

// NOTE: The Client list could alse be implemented as a DoubleLinkedList.
pub struct Client {
	id: ClientId,
	name: String,
	remote: Remote,
	pending_request: Option<String>,
	nethandler: Arc<RwLock<NetHandler>>
}

impl Client {
	pub fn new(id: usize, stream: TcpStream, handler: Arc<RwLock<NetHandler>>) -> Client {
		println!("New client. ID: [{}], IP: [{}]", id, stream.peer_addr().unwrap());

		Client {
			id: id,
			name: String::new(),
			remote: Remote::new(stream).expect("Could not establish connection"),
			pending_request: None,
			nethandler: handler
		}
	}

	pub fn name(&self) -> String {
		self.name.clone()
	}

	pub fn set_name(&mut self, name: String) {
		self.name = name
	}
}

pub fn change_client_name(client: Arc<Mutex<Client>>, new_name: String, client_map: &ClientMap) {
	client.lock().unwrap().set_name(new_name);
	client_map.broadcast();
}

pub fn handle_game_request(source: &Client, requestee: ClientId, client_map: Arc<Mutex<ClientMap>>) {
	let requestee = match client_map.get_by_name(&requestee) {
		Some(requestee) => requestee,
		None => return
	};
}
