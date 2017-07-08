use std::sync::{Arc, Mutex, Weak};
use std::net::TcpStream;
use super::{ClientId, ClientMap};
use packets::Packet;

// NOTE: The Client list could alse be implemented as a DoubleLinkedList.
pub struct Client {
	id: ClientId,
	stream: TcpStream,
	clients: Weak<Mutex<ClientMap>>
}

impl Client {
	pub fn new(id: usize, stream: TcpStream, clients: Weak<Mutex<ClientMap>>) -> Client {
		println!("New client. ID: [{}], IP: [{}]", id, stream.peer_addr().unwrap());

		Client {
			id: id,
			stream: stream,
			clients: clients
		}
	}

	pub fn send<P>(&mut self, p: &P) -> bool where P: Packet {
		p.write_to_stream(&mut self.stream)
	}

	pub fn stream(&self) -> &TcpStream {
		&self.stream
	}
}
