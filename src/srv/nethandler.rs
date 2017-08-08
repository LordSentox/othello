use std::sync::{Arc, mpsc, Mutex, RwLock, RwLockReadGuard, Weak};
use std::sync::mpsc::{Sender, Receiver};
use packets::*;
use remote::Remote;
use super::netclient::NetClient;
use std::thread;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::collections::HashMap;
use std::io::Error as IOError;
use std::sync::atomic::{AtomicBool, Ordering};

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
fn search_free_id<T>(map: &mut HashMap<ClientId, T>, start: ClientId) -> Option<ClientId> {
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
#[derive(Clone)]
pub struct NetHandler {
    clients: ArcRw<HashMap<ClientId, Arc<NetClient>>>,
	packets: Arc<Receiver<Packet>>,
	sender: Sender<Packet>,
	listening: Arc<AtomicBool>
}

impl NetHandler {
	pub fn new() -> NetHandler {
		let (sender, receiver) = mpsc::channel();

		NetHandler {
			clients: Arc::new(RwLock::new(HashMap::new())),
			packets: Arc::new(receiver),
			sender: sender,
			listening: Arc::new(AtomicBool::new(false))
		}
	}

    /// Start lisening and accepting clients. This action is non-blocking and starts another thread.
    pub fn start_listen(&self, port: u16) -> Result<(), Error> {
		if self.listening.load(Ordering::Relaxed) {
			return Err(Error::AlreadyListening);
		}

		// Create a new listener on the local address with the specified port.
		let listener = match TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
			Ipv4Addr::new(127, 0, 0, 1), port))) {
				Ok(listener) => listener,
				Err(err) => return Err(Error::SockErr(err))
		};

        let clients_clone = self.clients.clone();
		let sender_clone = self.sender.clone();
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
				last_id =  match search_free_id(&mut clients_lock, last_id+1) {
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
                        println!("Couldn't create remote for the client. Dropping.");
                        continue;
                    }
                };

				// Create the client from the remote and. The client will then start receiving.
				let client = NetClient::from_remote(clients_clone.clone(), last_id, remote, sender_clone.clone());

				// Add the client to the client map and add a local bus for everyone that only wants
				// to packets coming from this client.
                clients_lock.insert(last_id, Arc::new(client));

                println!("Client connected. ID: {}", last_id);
            }
        });

		self.listening.store(false, Ordering::Relaxed);
		Ok(())
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
            let succ = client.send(&packet);
            println!("Broadcast failed for client [{}]", id);
			one_failed |= !succ;
		}
		one_failed
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

    // TODO: If it comes up later, create a function to get the Remote object for an ID. This way,
    // the lookups in the map can be reduced, increasing efficiency. However, at the moment this
    // would only decrease readability, so be sure it is what you want.

}
