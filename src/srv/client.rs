use std::sync::{Arc, Weak, RwLock};
use std::thread;
use std::net::TcpStream;
use super::NetHandler;
use remote::Remote;
use packets::*;
use board::*;

/// A Game with another client.
struct Game {
	pub board: Arc<RwLock<Board>>,
	pub opponent: Weak<RwLock<Client>>,
	pub piece: Piece
}

pub struct Client {
	id: ClientId,
	name: String,
	remote: Remote,
	pending_request: Option<String>,
	game: Option<Game>,
	nethandler: Arc<RwLock<NetHandler>>
}

impl Client {
	/// Create a new Client. This will not register the client on the provided
	/// NetHandler. If you want to access multiple clients, please do that
	/// manually.
	pub fn new(id: ClientId, stream: TcpStream, handler: Arc<RwLock<NetHandler>>) -> Client {
		println!("New client. ID: [{}], IP: [{}]", id, stream.peer_addr().unwrap());

		Client {
			id: id,
			name: String::new(),
			remote: Remote::new(stream).expect("Could not establish connection"),
			pending_request: None,
			game: None,
			nethandler: handler
		}
	}

	/// Starts a new thread and begins receiving on it. The client will close,
	/// once the thread has been killed, or the connection has been closed by
	/// the remote host.
	pub fn start_receiving(client: Arc<RwLock<Client>>) {
		thread::spawn(move || {
			loop {
				// Read the newest packet. Make sure not to lock the client the
				// entire time after the packet has been read.
				let packet = {
					let client_lock = client.read().unwrap();
					client_lock.remote.read_packet()
				};

				match packet {
					Ok(p) => Client::handle_packet(client.clone(), p),
					Err(PacketReadError::Closed) => {
						// The connection has been closed. Remove the client from
						// the NetHandler and end the receiving stream.
						let client_lock = client.read().unwrap();
						let mut nethandler_lock = client_lock.nethandler.write().unwrap();
						println!("Client [{}] disconnected.", client_lock.id);
						nethandler_lock.unregister_client(client_lock.id);
						break;
					}
					Err(err) => println!("Error reading packet {:?}", err)
				};
			}
		});
	}

	/// Send a packet to the remote host represented by this client.
	pub fn write_packet(&self, p: &Packet) -> bool {
		self.remote.write_packet(&p)
	}

	/// Try to change the name of the client. Sends a response to the client
	/// to tell him, if the change has been successful.
	// NOTE: Should the implementation of NetHandler change, the client might
	// need to register the name change with its respective NetHandler.
	fn change_name(client: Arc<RwLock<Client>>, name: &String) {
		let successful = {
			let client_lock = client.read().unwrap();
			let successful = !client_lock.nethandler.read().unwrap().is_name_registered(&name);
			successful
		};

		// Send the response to the client.
		let p = Packet::ChangeNameResponse(successful);
		client.read().unwrap().remote.write_packet(&p);

		if successful {
			client.write().unwrap().name = name.clone();
		}
	}

	/// Send a packet containing all clients which are registered on the same
	/// NetHandler as this client to the clients remote connection.
	fn send_clients_to_peer(&self) {
		// Since the client is only locked in read mode, iterating through the
		// client list in read mode as well is okay.
		let p = Packet::ClientList(self.nethandler.read().unwrap().client_list());

		if !self.remote.write_packet(&p) {
			println!("Failed to send client [{}] the client list.", self.id);
		}
	}

	/// Request a game from the client with the provided name.
	/// Returns the requestee in case a game should be started, None otherwise.
	fn request_game(&mut self, target: &String) -> Option<Weak<RwLock<Client>>> {
		// TODO: The client should be informed, if the target client actually
		// does not exist, instead of this case just being ignored.

		let target_client = match self.nethandler.read().unwrap().get_by_name(&target) {
			Some(target) => target,
			None => return None
		};

		// The target exists, so register that the game should be played.
		self.pending_request = Some(target.clone());

		let target_client_arc = target_client.upgrade().unwrap();
		if Some(self.name.clone()) == target_client_arc.read().unwrap().pending_request {
			// The target has already requested a game from this client, so a
			// game should be started.
			Some(target_client)
		}
		else {
			None
		}
	}

	/// Handle a single packet. This is called every time, the remote instance
	/// has successfully read a packet from the receiving thread automatically.
	fn handle_packet(client: Arc<RwLock<Client>>, p: Packet) {
		println!("Handling packet {:?}", p);

		match p {
			Packet::ChangeNameRequest(new_name) => Client::change_name(client.clone(), &new_name),
			Packet::ChangeNameResponse(_) => println!("Packet [ChangeNameResponse] is only valid in direction Server->Client."),
			Packet::RequestClientList => client.read().unwrap().send_clients_to_peer(),
			Packet::ClientList(_) => println!("Packet [ClientList] is only valid in direction Server->Client."),
			Packet::RequestGame(requestee_name) => {
				if let Some(requestee) = client.write().unwrap().request_game(&requestee_name) {
					// Start the game between the two clients.

				}
			},
			Packet::StartGame(_) => println!("Packet [StartGame] is only valid in direction Server->Client")
		}
	}

	/// The internal id of the client used by the NetHandler.
	pub fn id(&self) -> ClientId {
		self.id
	}

	/// The name of the client.
	pub fn name(&self) -> String {
		self.name.clone()
	}
}
