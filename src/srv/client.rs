use std::sync::{Arc, Mutex, Weak};
use std::net::TcpStream;
use super::{ClientId, ClientMap};
use packets::Packet;

// NOTE: The Client list could alse be implemented as a DoubleLinkedList.
pub struct Client {
	id: ClientId,
	name: String,
	stream: TcpStream,
	clients: Weak<Mutex<ClientMap>>,
	pending_request: Option<String>
}

impl Client {
	pub fn new(id: usize, stream: TcpStream, clients: Weak<Mutex<ClientMap>>) -> Client {
		println!("New client. ID: [{}], IP: [{}]", id, stream.peer_addr().unwrap());

		Client {
			id: id,
			name: String::new(),
			stream: stream,
			clients: clients
		}
	}

	pub fn name(&self) -> String {
		self.name.clone()
	}

	pub fn set_name(&mut self, name: String) {
		self.name = name
	}

	pub fn stream(&self) -> &TcpStream {
		&self.stream
	}

	pub fn stream_mut(&mut self) -> &mut TcpStream {
		&mut self.stream
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
