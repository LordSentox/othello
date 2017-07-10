use std::sync::{Arc, RwLock, Weak};
use std::thread;
#[macro_use]
use packets::*;
use super::client::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::io::{Error as IOError, Read};
use std::collections::HashMap;

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
			let mut last_id = 0;

			for stream in listener.incoming() {
				// Check if the stream is valid and try to create a client for it.
				let stream = match stream {
					Ok(stream) => stream,
					Err(err) => {
						println!("Client tried to connect, but could not be accepted.");
						continue;
					}
				};

				let mut nethandler_lock = nethandler.write().unwrap();
				last_id =  match nethandler_lock.search_free_id(Some(last_id + 1)) {
					Some(id) => id,
					None => { println!("Could not find a free id. Denying client."); continue; }
				};

				let client = Arc::new(RwLock::new(Client::new(last_id, stream, nethandler.clone())));

				// Add the client to the NetHandler, then start the client stream
				// in a seperate thread.
				nethandler_lock.register_client(last_id, Arc::downgrade(&client));

				Client::start_receiving(client);
			}
		});
	}

	/// Returns the first client with the name provided, or None, if none could
	/// be found. This is a linear search, so use sparingly, if the NetHandler
	/// handles many clients or consider creating another cross-reference.
	pub fn get_by_name(&self, name: &String) -> Option<Weak<RwLock<Client>>> {
		for (_, ref client) in &self.clients {
			let client_arc = client.upgrade().unwrap();
			if client_arc.read().unwrap().name() == *name {
				return Some(Arc::downgrade(&client_arc));
			}
		}

		None
	}

	/// Returns a Packet::ClientList, containing all client currently registered
	/// on this NetHandler.
	pub fn client_list(&self) -> Vec<(ClientId, String)> {
		let mut vec: Vec<(ClientId, String)> = Vec::with_capacity(self.clients.len());
		for (id, ref client) in &self.clients {
			let client_arc = client.upgrade().unwrap();
			vec.push((id.clone(), client_arc.read().unwrap().name()));
		}
		vec
	}

	/// Send the packet provided to all clients registered on this NetHandler.
	pub fn broadcast(&self, p: &Packet) -> bool {
		let mut one_failed = false;
		for (_, ref client) in &self.clients {
			let client_arc = client.upgrade().unwrap();
			one_failed |= !client_arc.read().unwrap().write_packet(&p);
		}
		one_failed
	}

	/// Register a new client on this NetHandler. Note that the NetHandler does
	/// not increase the reference count, so the client can be removed freely.
	/// The NetHandler should be notified however, so that the client can be
	/// removed gracefully and doesn't cause any problems.
	pub fn register_client(&mut self, id: ClientId, client: Weak<RwLock<Client>>) {
		assert_eq!(self.clients.contains_key(&id), false);

		self.clients.insert(id, client);
	}

	/// Remove the client from the registry of the NetHandler. This is not
	/// strictly necessary, but recommended, since not doing this might result
	/// in multiple problems.
	/// This function must be called, after the client has actually been removed.
	pub fn unregister_client(&mut self, id: ClientId) {
		// Check that the client really doesn't exist anymore.

		self.clients.remove(&id);
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
