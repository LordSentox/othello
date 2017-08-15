use std::sync::{Arc, Weak, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use packets::*;
use remote::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::collections::VecDeque;
use std::io::Error as IOError;
use std::io::ErrorKind as IOErrorKind;
use std::time::Duration;

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
	remote: Arc<Remote>,
	packets: ArcRw<Vec<Weak<Mutex<VecDeque<Packet>>>>>,
	handle: Option<JoinHandle<()>>,
	running: Arc<AtomicBool>
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

		// Now that this sequence is over, a timeout is set on the read thread, so that it can be
		// cancelled gracefully.
		remote.set_timeout(Some(Duration::from_millis(500)), DirSocket::Read).expect("Could not set Socket read timeout.");
		let remote = Arc::new(remote);
		let packets: ArcRw<Vec<Weak<Mutex<VecDeque<Packet>>>>> = Arc::new(RwLock::new(Vec::new()));

		// The connection has been established successfully. The login was successful.
		// Now we can start the Network-thread and return the NetHandler.
		let remote_clone = remote.clone();
		let packets_clone = packets.clone();
		let running = Arc::new(AtomicBool::new(true));
		let running_clone = running.clone();
		let handle = thread::spawn(move || {
			while running_clone.load(Ordering::Relaxed) {
				let packet = match remote_clone.read_packet() {
					Ok(p) => p,
					Err(PacketReadError::Closed) => {
						println!("The connection has been closed by the server.");
						Packet::Disconnect
					},
					Err(PacketReadError::IOError(err)) => {
						if let IOErrorKind::WouldBlock = err.kind() {
							// This error is to be expected and can be ignored.
							continue;
						}

						println!("Error reading packet: {:?}", err);
						continue;
					}
					Err(err) => {
                        // An error occured. Ignore this packet.
                        println!("Error reading packet. {:?}", err);
                        continue;
					}
				};

				// Send the packet to all subscribed handlers.
				for s in &*packets_clone.read().unwrap() {
					if let Some(s) = s.upgrade() {
						s.lock().unwrap().push_back(packet.clone());
					}
				}

				println!("Packet received: {:?}", packet.clone());

                // If the Disconnection packet has been created, there will no longer be anything
                // to do, so the nethandler will be stopped.
                if let Packet::Disconnect = packet {
                    running_clone.store(false, Ordering::Relaxed);
                }
			}
		});

		Ok(Arc::new(NetHandler {
			client_id: id,
			login_name: login_name.to_string(),
			remote: remote,
			packets: packets,
			handle: Some(handle),
			running: running
		}))
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

	/// Returns true if the NetHandler is still connected to the server, otherwise false.
	pub fn connected(&self) -> bool {
		self.running.load(Ordering::Relaxed)
	}
}

impl Drop for NetHandler {
	fn drop(&mut self) {
		self.running.store(false, Ordering::Relaxed);

		self.handle.take().unwrap().join().expect("Could not join packet receiving thread.");
	}
}
