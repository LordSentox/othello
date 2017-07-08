use std::sync::{Arc, Mutex, Weak};
use std::thread;
use packets::*;
use super::Client;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::io;
use std::collections::HashMap;

pub type ClientId = usize;
use std::usize::MAX as ClientIdMAX;

pub type ClientMap = HashMap<ClientId, Client>;

pub struct NetHandler {
	listener: TcpListener,
	clients: Arc<Mutex<ClientMap>>
}

fn search_free_id<T>(start: ClientId, map: &HashMap<ClientId, T>) -> Option<ClientId> {
	// Search high, since this is more probable.
	for key in start..ClientIdMAX {
		if !map.contains_key(&key) {
			return Some(key);
		}
	}

	// Search low, since some old keys might be free again.
	for key in 0..start - 1 {
		if !map.contains_key(&key) {
			return Some(key);
		}
	}

	None
}

impl NetHandler {
	pub fn new(port: u16) -> Result<NetHandler, io::Error> {
		let listener = match TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
								Ipv4Addr::new(127, 0, 0, 1), port))) {
			Ok(listener) => listener,
			Err(err) => return Err(err)
		};

		Ok(NetHandler {
			listener: listener,
			clients: Arc::new(Mutex::new(ClientMap::new()))
		})
	}

	pub fn listen(&mut self) -> ! {
		// A point to start looking for ids that have not been taken yet.
		let mut start_id = 0;

		loop {
			for stream in self.listener.incoming() {
				match stream {
					Ok(stream) => {
						// Assign an unused id to the new client.
						{
							let mut clients_lock = self.clients.lock().unwrap();
							start_id = match search_free_id(start_id + 1, &clients_lock) {
								Some(id) => id,
								None => { println!("Could not find a free id. Denying client."); continue; }
							};
							let new_client = Client::new(start_id,
												stream.try_clone().expect("Could not clone stream, which is critical."),
												Arc::downgrade(&self.clients));
							clients_lock.insert(start_id, new_client);
						}
						// This is an asynchronous call. It reads all the packets for the given
						// client id. Every client has it's own read thread this way.
						self.handle_incoming_packets(start_id);
					},
					Err(err) => println!("Client failed to establish connection: {}", err)
				}
			}
		}
	}

	fn handle_incoming_packets(&self, id: ClientId) {
		let clients_lock = self.clients.lock().unwrap();
		let client: &Client = clients_lock.get(&id).unwrap();

		let stream_clone = client.stream().try_clone().expect("Failed to clone stream, which is critical.");
		let clients_map_clone = self.clients.clone();

		thread::spawn(move || {
			
		});
	}
}

pub trait NetClientMap {
	fn broadcast<P>(&mut self, p: &P) -> bool where P: Packet;
}

impl NetClientMap for ClientMap {
	fn broadcast<P>(&mut self, p: &P) -> bool where P: Packet {
		let mut one_failed = false;
		for (ref id, ref mut client) in self.iter_mut() {
			one_failed |= !client.send(p)
		}
		one_failed
	}
}
