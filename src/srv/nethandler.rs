use std::sync::{Arc, mpsc, Mutex, RwLock, RwLockReadGuard, Weak};
use std::sync::mpsc::{Sender, Receiver};
use packets::*;
use remote::Remote;
use super::netclient::NetClient;
use std::thread;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::collections::HashMap;
use std::io::Error as IOError;
use bus::{Bus, BusReader};

pub type ArcRw<T> = Arc<RwLock<T>>;

/// Look for any id that has not been given to a client. Optionally,
/// a starting id can be provided, where it is expected there is room
/// close after it. If None is given, it starts with 0.
fn search_free_id<T>(map: &mut HashMap<ClientId, T>, start: Option<ClientId>) -> Option<ClientId> {
	let start = match start {
		Some(start) => start,
		None => 0
	};

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

/// Listens for clients and accepts them. The packets can then be queried by another thread.
/// The NetHandler is designed to be shared immutably between any number of threads.
pub struct NetHandler {
    clients: ArcRw<HashMap<ClientId, Arc<NetClient>>>,
	global_bus: ArcRw<Bus<(ClientId, Packet)>>
}

impl NetHandler {
	pub fn new() -> NetHandler {
		NetHandler {
			clients: Arc::new(RwLock::new(HashMap::new())),
			global_bus: Arc::new(Mutex::new(Bus::new(10)))
		}
	}

    /// Start lisening and accepting clients. This action is non-blocking and starts another thread.
    pub fn start_listen(&self, port: u16) -> Result<(), IOError> {
		// Create a new listener on the local address with the specified port.
		let listener = match TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
			Ipv4Addr::new(127, 0, 0, 1), port))) {
				Ok(listener) => listener,
				Err(err) => return Err(err)
		};

        let clients_clone = self.clients.clone();
		let global_bus_clone = self.global_bus.clone();
		let client_busses_clone = self.client_busses.clone();
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

				let mut clients_lock = clients_clone.write().unwrap();
				last_id =  match search_free_id(&mut clients_lock, Some(last_id + 1)) {
					Some(id) => id,
					None => {
                        println!("Could not find a free id. Denying client.");
                        continue;
                    }
				};

				// Wrap the stream in a remote and add it to the client map. Then start receiving
                // for that client.
                let remote = Arc::new(match Remote::new(stream) {
                    Ok(r) => r,
                    Err(err) => {
                        println!("Couldn't create remote for the client. Dropping.");
                        continue;
                    }
                });

				// Add the client to the client map and add a local bus for everyone that only wants
				// to packets coming from this client.
                clients_lock.insert(last_id, remote.clone());
				client_busses_clone.write().unwrap().insert(last_id, Mutex::new(Bus::new(10)));

                println!("Client connected. ID: {}", last_id);

                // Start receiving the packets for the client.
            }
        });
		Ok(())
    }

	/// Subscribe to the packet channel of this NetHandler. Whenever a packet is received, it will
	/// be pushed to all subscribed threads.
	// TODO: In the future, it may be wise to change this to somehow only push packets of a specified
	// type, to not multiply all packets needlessly handeling packets on all subscribers.
	pub fn subscribe_all(&self) -> BusReader<(ClientId, Packet)> {
		// Get another reader from the bus.
		self.global_bus.lock().unwrap().add_rx()
	}

	/// Subscribe to a single client.

    /// Checks, if a client with the id exists and returns true if it does.
    pub fn has_client(&self, client: ClientId) -> bool {
        self.clients.read().unwrap().contains_key(&client)
    }

    /// Broadcast a packet to all clients. Returns false, if the packet could not be broadcast to
    /// all clients, i.e. if at least one send operation failed.
    pub fn broadcast(&self, packet: &Packet) -> bool {
		let mut one_failed = false;
        let clients = self.clients.read().unwrap();
		for (ref id, ref remote) in &*clients {
            let succ = remote.write_packet(&packet);
            println!("Broadcast failed for client [{}]", id);
			one_failed |= !succ;
		}
		one_failed
    }

    /// Sends a packet to the client with the corresponding client id and returns true, if the
    /// packet has been sent successfully, false otherwise.
    pub fn send(&self, client: ClientId, packet: &Packet) -> bool {
        match self.clients.read().unwrap().get(&client) {
            Some(ref remote) => remote.write_packet(packet),
            None => {
                println!("Attempted access of invalid client id. [{}]", client);
                return false;
            }
        }
    }

    // TODO: If it comes up later, create a function to get the Remote object for an ID. This way,
    // the lookups in the map can be reduced, increasing efficiency. However, at the moment this
    // would only decrease readability, so be sure it is what you want.

}
