use std::sync::{Arc, Weak, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use packets::*;
use remote::Remote;
use std::net::{TcpStream, ToSocketAddrs};
use std::collections::VecDeque;
use std::io::Error as IOError;

pub type ArcRw<T> = Arc<RwLock<T>>;

#[derive(Debug)]
pub enum Error {
	Refused,
	SendLoginFailed,
	LoginDeny(String),
	ProtocolError,
	PacketRead(PacketReadError),
	SockErr(IOError)
}

pub struct NetHandler {
	client_id: ClientId,
	login_name: String,
	remote: Remote,
	packets: RwLock<Vec<Weak<Mutex<VecDeque<Packet>>>>>,
	// pt_handle: Option<JoinHandle<()>>
}

impl NetHandler {
	pub fn connect<A: ToSocketAddrs>(addrs: A, login_name: &str) -> Result<Arc<NetHandler>, Error> {
		// Try to connect to the server.
		let stream = match TcpStream::connect(addrs) {
			Ok(stream) => stream,
			Err(err) => return Err(Error::SockErr(err))
		};

		let remote = match Remote::new(stream) {
			Ok(remote) => remote,
			Err(err) => return Err(Error::SockErr(err))
		};

		// The connection has been established. Let's wait if the server responds with a login
		// accept.
		let id = match remote.read_packet() {
			Ok(Packet::ConnectSuccess(id)) => id,
			Ok(p) => {
				println!("Received unexpected packet {:?}, expected Packet::ConnectSuccess", p);
				return Err(Error::ProtocolError);
			},
			Err(err) => return Err(Error::PacketRead(err))
		};

		// The connection has been established. Now try to login with the provided
		// Login name.
		if !remote.write_packet(&Packet::Login(login_name.to_string())) {
			return Err(Error::SendLoginFailed);
		}

		// Sent the login request successfully. Now await the response from the server.
		match remote.read_packet() {
			Ok(Packet::LoginAccept) => println!("Logged in as {}", login_name),
			Ok(Packet::LoginDeny(reason)) => return Err(Error::LoginDeny(reason)),
			Ok(p) => {
				println!("Received unexpected packet {:?}, expected LoginDeny or LoginAccept packet.", p);
				return Err(Error::ProtocolError);
			},
			Err(err) => return Err(Error::PacketRead(err))
		}

		let nethandler = Arc::new(NetHandler {
			client_id: id,
			login_name: login_name.to_string(),
			remote: remote,
			packets: RwLock::new(Vec::new()),
			// pt_handle: None
		});

		// The connection has been established successfully. The login was successful.
		// Now we can start the Network-thread and return the NetHandler.
		let nethandler_clone = nethandler.clone();
		let _ = thread::spawn(move || {
			loop {
				let packet = match nethandler_clone.remote.read_packet() {
					Ok(p) => p,
					Err(PacketReadError::Closed) => {
						println!("The connection has been closed by the server.");
						Packet::Disconnect
					},
					Err(err) => {
                        // An error occured. Ignore this packet.
                        println!("Error reading packet from client [{}]. {:?}", id, err);
                        continue;
					}
				};

				// Send the packet to all subscribed handlers.
				for s in &*nethandler_clone.packets.read().unwrap() {
					if let Some(s) = s.upgrade() {
						s.lock().unwrap().push_back(packet.clone());
					}
				}

				println!("Packet received: {:?}", packet.clone());

                // If the Disconnection packet has been created, there will no longer be anything
                // to do, so the nethandler will be stopped.
                if let Packet::Disconnect = packet {
                    break;
                }
			}
		});

		Ok(nethandler)
	}

	/// Send a packet to the server.
	pub fn send(&self, p: &Packet) -> bool {
		self.remote.write_packet(&p)
	}

	/// Subscribe and receive all packets from the server. They are saved into the packets-VecDeque
	/// provided.
	pub fn subscribe(&self, packets: Weak<Mutex<VecDeque<Packet>>>) {
		self.packets.write().unwrap().push(packets)
	}

    /// The internal id of the client used by the NetHandler.
    pub fn id(&self) -> ClientId {
        self.client_id
    }

	/// Get the login name of the client used by the Master Server.
	pub fn login_name(&self) -> String {
		self.login_name.clone()
	}
}
