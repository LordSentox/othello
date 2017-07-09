use std::sync::{Arc, Mutex, Weak};
use std::thread;
#[macro_use]
use packets::*;
use super::Client;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::io::{Error as IOError, Read};
use std::collections::HashMap;

pub type ClientId = usize;
use std::usize::MAX as ClientIdMAX;

pub type ClientMap = HashMap<ClientId, Arc<Mutex<Client>>>;

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
	pub fn new(port: u16) -> Result<NetHandler, IOError> {
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
						let new_client = {
							let mut clients_lock = self.clients.lock().unwrap();
							start_id = match search_free_id(start_id + 1, &clients_lock) {
								Some(id) => id,
								None => { println!("Could not find a free id. Denying client."); continue; }
							};
							let new_client = Arc::new(Mutex::new(Client::new(start_id,
												stream.try_clone().expect("Could not clone stream, which is critical."),
												Arc::downgrade(&self.clients))));
							clients_lock.insert(start_id, new_client.clone());
							new_client // Extend the lifetime of the new client
						};

						// This is an asynchronous call. It reads all the packets for the given
						// client id. Every client has it's own read thread this way.
						self.handle_incoming_packets(start_id, new_client);
					},
					Err(err) => println!("Client failed to establish connection: {}", err)
				}
			}
		}
	}

	fn handle_incoming_packets(&self, cid: ClientId, client: Arc<Mutex<Client>>) {
		let mut stream = client.lock().unwrap().stream().try_clone().expect("Failed to clone stream, which is critical.");
		let clients_map = self.clients.clone();

		thread::spawn(move || { loop {
			// Read the packet id or detect that the stream has been closed and clean up.
			let mut pid = vec![0; 1];
			let len = match stream.read(&mut pid) {
				Ok(0) => {
					// The stream has been closed.
					println!("Connection has been closed by the client.");

					let mut lock = clients_map.lock().unwrap();
					lock.remove(&cid);
					break;
				},
				Ok(len) => len,
				Err(err) => {
					println!("Error reading packet id: {}", err);
					continue;
				}
			};

			assert_eq!(len, 1);
			let pid = pid[0];

			let packet = read_packet!(pid, stream);
		}});
	}
}

pub trait NetClientMap {
	fn broadcast<P>(&mut self, p: &P) -> bool where P: Packet;
}

impl NetClientMap for ClientMap {
	fn broadcast<P>(&mut self, p: &P) -> bool where P: Packet {
		let mut one_failed = false;
		for (ref id, ref mut client) in self.iter_mut() {
			one_failed |= !client.lock().unwrap().send(p)
		}
		one_failed
	}
}
