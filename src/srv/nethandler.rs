use std::sync::{Arc, RwLock, Weak};
use std::thread;
#[macro_use]
use packets::*;
use super::client::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::io::{Error as IOError, Read};
use std::collections::HashMap;

pub type ClientId = usize;
use std::usize::MAX as ClientIdMAX;

pub struct NetHandler {
	clients: HashMap<ClientId, Weak<RwLock<Client>>>
}

impl NetHandler {
	/// Create a new NetHandler and start listening on the specified port.
	/// If 0 is specified as the port, the OS will be asked to assign one.
	pub fn new(port: u16) -> Result<Arc<RwLock<NetHandler>>, IOError> {
		// Create a new listener on the local address with the specified port.
		let listener = match TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
			Ipv4Addr::new(127, 0, 0, 1), port))) {
				Ok(listener) => listener,
				Err(err) => return Err(err)
		};

		let nethandler = Arc::new(RwLock::new(NetHandler {
			clients: HashMap::new()
		}));

		NetHandler::start_listening(nethandler.clone(), listener);
		Ok(nethandler)
	}

	/// Starts a new thread that will listen to new incoming clients. They will then be added to
	/// the NetHandler and their client thread will be started. In other words, this is the main
	/// thread of the program where all others except for the control thread diverge from.
	fn start_listening(nethandler: Arc<RwLock<NetHandler>>, listener: TcpListener) {
		thread::spawn(move || {
			for stream in listener.incoming() {
				// Check if the stream is valid and try to create a client for it.
				let stream = match stream {
					Ok(stream) => stream,
					Err(err) => {
						println!("Client tried to connect, but could not be accepted.");
						continue;
					}
				};


			}
		});
	}

	/// Look for any id that has not been given to a client. Optionally,
	/// a starting id can be provided, where it is expected there is room
	/// close after it. If None is given, it starts with 0.
	pub fn search_free_id(&self, start: Option<ClientId>) -> Option<ClientId> {
		let start = match start {
			Some(start) => start,
			None => 0
		};

		// Search high, since this is more probable.
		for key in start..ClientIdMAX {
			if !self.clients.contains_key(&key) {
				return Some(key);
			}
		}

		// Search low, since some old keys might be free again.
		for key in 0..start - 1 {
			if !self.clients.contains_key(&key) {
				return Some(key);
			}
		}

		None
	}
}

impl NetHandler {
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
			// Read the newest packet sent by the client
			match read_from_stream(&mut stream) {
				(_, true) => {
					clients_map.lock().unwrap().remove(&cid);
					break;
				},
				(Some(p), false) => {
					// Packet has been received. It will be handled accordingly.
					handle_packet(&p, clients_map.clone(), client.clone(), cid);
				}
			}
		}});
	}
}

/*
pub trait ClientMap {
	pub fn get_by_name(&self, name: &String) -> Option<&Client>;
}

impl ClientMap {
	/// Performs a linear search to find the first client with this name.
	pub fn get_by_name(&self, name: &String) -> Option<&Client> {
		for (_, client) in self {
			let lock = client.lock().unwrap();
			if lock.name() == name {
				return Some(&lock);
			}
		}

		None
	}

	pub fn broadcast(&self) {
		broadcast(&Packet::ClientList(self.to_name_vec()), &self);
	}

	pub fn to_name_vec(&self) -> Vec<(u64, String)> {
		let mut vec = Vec::new();
		for (ref id, ref client) in self {
			vec.push((id, client.lock().unwrap().name()));
		}

		vec
	}
}*/

/// Broadcast the message to all clients in the map given.
/// Returns true, if the message has been sent to all clients successfully, false otherwise.
pub fn broadcast(p: &Packet, clients: &ClientMap) -> bool {
	let mut one_failed = false;
	for client in clients {
		one_failed |= !write_to_stream(&p, client.stream_mut());
	}
	one_failed
}

/// All incoming packets go through here. They are then potentially distributed to other places.
fn handle_packet(p: &Packet, client_map: Arc<Mutex<ClientMap>>, client: Arc<Mutex<Client>>, cid: ClientId) {
	match p {
		Packet::ChangeNameRequest(new_name) => change_client_name(client, new_name, client_map.lock().unwrap()),
		Packet::RequestClientList => write_to_stream(&Packet::ClientList(client_map.lock().unwrap().to_name_vec()), client.lock().unwrap().stream_mut()),
		Packet::ClientList(_) => println!("Packet [ClientList] is only valid in direction Server->Client"),
		Packet::RequestGame(requestee) => handle_game_request(client.lock().unwrap(), requestee, client_map)
	}
}
