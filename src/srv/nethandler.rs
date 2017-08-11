use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
use packets::*;
use remote::Remote;
use super::netclient::NetClient;
use std::thread;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::collections::{HashMap, VecDeque};
use std::io::Error as IOError;

pub type ArcRw<T> = Arc<RwLock<T>>;
pub const SERVER_ID: ClientId = 0;

#[derive(Debug)]
pub enum Error {
	AlreadyListening,
	SockErr(IOError)
}

/// Look for any id that has not been given to a client. Optionally,
/// a starting id can be provided, where it is expected there is room
/// close after it. If None is given, it starts with 0.
fn search_free_id<T>(map: &HashMap<ClientId, T>, start: ClientId) -> Option<ClientId> {
	// Search high, since this is more probable.
	for key in start..ClientIdMAX {
		if !map.contains_key(&key) {
			return Some(key);
		}
	}

	// Search low, since some old keys might be free again.
	for key in 1..start - 1 {
		if !map.contains_key(&key) {
			return Some(key);
		}
	}

	None
}

/// Listens for clients and accepts them. The packets can then be queried by another thread.
/// The NetHandler is designed to be cloned and shared between any number of threads.
pub struct NetHandler {
    clients: RwLock<HashMap<ClientId, Arc<NetClient>>>,
	packets: RwLock<Vec<Weak<Mutex<VecDeque<(ClientId, Packet)>>>>>
}

impl NetHandler {
    /// Start lisening and accepting clients. This action is non-blocking and starts another thread.
    pub fn start_listen(port: u16) -> Result<Arc<NetHandler>, Error> {
		// Create a new listener on the local address with the specified port.
		let listener = match TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
			Ipv4Addr::new(0, 0, 0, 0), port))) {
				Ok(listener) => listener,
				Err(err) => return Err(Error::SockErr(err))
		};

		let nethandler = Arc::new(NetHandler {
			clients: RwLock::new(HashMap::new()),
			packets: RwLock::new(Vec::new())
		});

		let self_clone = nethandler.clone();
        thread::spawn(move || {
			let mut last_id: ClientId = 0;

			// Listen for the next client that tries to connect.
			for stream in listener.incoming() {
				// Check if the stream is valid and try to create a client for it.
				let stream = match stream {
					Ok(stream) => stream,
					Err(err) => {
						println!("Client tried to connect, but could not be accepted. {}", err);
						continue;
					}
				};

				last_id =  match search_free_id(&self_clone.clients(), last_id+1) {
					Some(id) => id,
					None => {
                        println!("Could not find a free id. Denying client.");
                        continue;
                    }
				};

				// Wrap the stream in a remote and add it to the client map. Then start receiving
                // for that client.
                let remote = match Remote::new(stream) {
                    Ok(r) => r,
                    Err(err) => {
                        println!("Couldn't create remote for the client. Dropping. {:?}", err);
                        continue;
                    }
                };

				// Create the client from the remote and. The client will then start receiving.
				let client = NetClient::start(self_clone.clone(), last_id, remote);

				// Add the client to the client map and add a local bus for everyone that only wants
				// to packets coming from this client.
                self_clone.clients_mut().insert(last_id, Arc::new(client));

                println!("Client connected. ID: {}", last_id);

				// Let the client know which id it will be referred to after this.
				self_clone.send(last_id, &Packet::ConnectSuccess(last_id));
            }
        });

		Ok(nethandler)
    }

    /// Checks, if a client with the id exists and returns true if it does.
    pub fn has_client(&self, client: ClientId) -> bool {
        self.clients.read().unwrap().contains_key(&client)
    }

    /// Broadcast a packet to all clients. Returns false, if the packet could not be broadcast to
    /// all clients, i.e. if at least one send operation failed.
    pub fn broadcast(&self, packet: &Packet) -> bool {
		let mut one_failed = false;
        let clients = self.clients.read().unwrap();
		for (ref id, ref client) in &*clients {
            if !client.send(&packet) {
	            println!("Broadcasting {:?} failed for client [{}]", packet, id);
				one_failed = true;
			}
		}
		!one_failed
    }

    /// Sends a packet to the client with the corresponding client id and returns true, if the
    /// packet has been sent successfully, false otherwise.
    pub fn send(&self, client: ClientId, packet: &Packet) -> bool {
        match self.clients.read().unwrap().get(&client) {
            Some(ref client) => client.send(&packet),
            None => {
                println!("Attempted access of invalid client id. [{}]", client);
                return false;
            }
        }
    }

	/// Get a weak reference to the client. Should they disconnect, the parent reference is
	/// dropped, so it is impossible to predict, how long the client will stay available.
	pub fn get_client(&self, id: ClientId) -> Option<Weak<NetClient>> {
		match self.clients.read().unwrap().get(&id) {
			Some(c) => Some(Arc::downgrade(c)),
			None => None
		}
	}

	/// Get an immutable list of all clients currently connected to this NetHandler.
	pub fn clients(&self) -> RwLockReadGuard<HashMap<ClientId, Arc<NetClient>>> {
		self.clients.read().unwrap()
	}

	/// Get a mutable list of all clients currently connected to this NetHandler.
	pub (super) fn clients_mut(&self) -> RwLockWriteGuard<HashMap<ClientId, Arc<NetClient>>> {
		self.clients.write().unwrap()
	}

	/// Subscribe to the packet channel of this NetHandler. The Weak VecDeque is where the packets
	/// are pushed.
	pub fn subscribe(&self, packets: Weak<Mutex<VecDeque<(ClientId, Packet)>>>) {
		self.packets.write().unwrap().push(packets)
	}

	pub fn subscribe_to(&self, client: ClientId, packets: Weak<Mutex<VecDeque<Packet>>>) -> bool {
		if let Some(client) = self.get_client(client) {
			if let Some(client) = client.upgrade() {
				client.subscribe(packets);
				true
			}
			else { false }
		}
		else { false }
	}

	/// Push a packet to the global VecDeque(s) so that everyone who has subscribed to those can
	/// handle the packet.
	pub (super) fn push_packet(&self, id: ClientId, packet: Packet) {
		assert!(self.has_client(id));

		// Add the packet to all VecDeques currently registered.
		for s in &*self.packets.read().unwrap() {
			if let Some(s) = s.upgrade() {
				s.lock().unwrap().push_back((id, packet.clone()));
			}
		}

		// If the packets can be written to, remove all the dead Weak pointers. This has to just
		// work sometimes, but it could still theoretically be starved, so:
		// TODO: Check, if this is starved and create a dedicated function if so.
		if let Ok(mut packets) = self.packets.try_write() {
			packets.retain(|ref s| {s.upgrade().is_some()});
		}

		// Check if the packet was a disconnect packet and remove the client from the register if so.
		if let Packet::Disconnect = packet {
			self.clients.write().unwrap().remove(&id);
		}
	}
}
