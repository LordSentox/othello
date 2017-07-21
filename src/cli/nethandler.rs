use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use std::sync::{Arc, Weak};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::thread;
use std::thread::JoinHandle;
use remote::*;
use std::io::Error as IOError;
use packets::*;

#[derive(Debug)]
pub enum Error {
	IO(IOError),
	NameChangeFailed,
	Receive(PacketReadError)
}

pub struct NetHandler {
	remote: Arc<Remote>,
	rcv_handle: Option<JoinHandle<()>>,
	p_rcv: Option<Receiver<Packet>>,
	last_cli_list: Vec<(ClientId, String)>
}

impl NetHandler {
	/// Attempts to connect to a server on the given address with the provided
	/// name.
	/// Returns a NetHandler on success. The NetHandler will however not start
	/// receiving packets until explicitly asked.
	pub fn new<A>(addrs: A, name: &str) -> Result<NetHandler, Error> where A: ToSocketAddrs {
		// Try to connect to the server.
		let stream = match TcpStream::connect(addrs) {
			Ok(stream) => stream,
			Err(err) => return Err(Error::IO(err))
		};

		let remote = match Remote::new(stream) {
			Ok(remote) => remote,
			Err(err) => { return Err(Error::IO(err)); }
		};

		// The connection has been established. Now we will change the name of
		// the client on the server, so that other clients can identify us easier.
		let p = Packet::ChangeNameRequest(name.to_string());
		if !remote.write_packet(&p) {
			return Err(Error::NameChangeFailed);
		}

		// Checkt that it has worked.
		// TODO: Currently there is no timeout on this, which could cause problems,
		// even though if the server closes the connection it does end.
		loop {
			match remote.read_packet() {
				Ok(Packet::ChangeNameResponse(true)) => {
					println!("Name has been set on the server.");
					break;
				},
				Ok(Packet::ChangeNameResponse(false)) => {
					println!("Name request has been denied by the server.");
					return Err(Error::NameChangeFailed);
				}
				Ok(p) => println!("Received unexpected packet from server: {:?} .. ignoring", p),
				Err(err) => return Err(Error::Receive(err))
			}
		}

		Ok(NetHandler {
			remote: Arc::new(remote),
			rcv_handle: None,
			p_rcv: None,
			last_cli_list: Vec::new()
		})
	}

	/// Start receiving packets for this NetHandler. This is a nonblocking operation. The packets
	/// which will be received can be handled in any thread the programmer deems right.
	pub fn start_receiving(&mut self) {
		self.remote.set_timeout(Some(Duration::from_secs(2)), DirSocket::READ).expect("Could not set socket timeout.");

		let remote: Weak<Remote> = Arc::downgrade(&self.remote);
		let (p_snd, p_rcv) = mpsc::channel::<Packet>();
		self.p_rcv = Some(p_rcv);

		self.rcv_handle = Some(thread::spawn(move || {
			while let Some(r) = remote.upgrade() {
				match r.read_packet() {
					Ok(p) => p_snd.send(p).unwrap(),
					Err(PacketReadError::Closed) => {
						println!("The server has closed the connection.");
						break;
					},
					Err(err) => println!("Error receiving packet. {:?}", err)
				}
			}
		}));
	}

	/// Handle all packets that have come in until the last call of this function or all packets
	/// until the time used up by this function exceeds the timeout duration.
	/// Note that if you use a timeout it is possible that packets build up over time, so it might
	/// be wise to not use a timeout from time to time when there is more time to spare.
	pub fn handle_packets(&mut self, timeout: Option<Duration>) {
		if self.p_rcv.is_none() { return; }

		if timeout.is_some() {
			let end_time = Instant::now() + timeout.unwrap();
			while Instant::now() < end_time {
				if !self.try_recv_and_handle() {
					return;
				}
			}
		}
		else {
			while self.try_recv_and_handle() {}
		}
	}

	#[inline]
	fn try_recv_and_handle(&mut self) -> bool {
		let p_rcv: &Receiver<Packet> = self.p_rcv.as_ref().unwrap();

		let p = match p_rcv.try_recv() {
			Ok(p) => p,
			Err(err) => { println!("Error handling packet. {}", err); return false; }
		};

		match p {
			Packet::ChangeNameRequest(_) => println!("Received invalid packet from Server. ChangeNameRequest is only valid in direction Client -> Server."),
			Packet::ChangeNameResponse(_) => println!("Received invalid packet from Server. ChangeNameResponse should only be received once at application startup."),
			Packet::RequestClientList => println!("Received invalid packet from Server. RequestClientList is only valid in direction Client -> Server."),
			Packet::ClientList(clients) => {
				println!("Clients currently on the server: ");
				for &(_, ref name) in &clients {
					println!("{}", name);
				}

				// Update to the newest client list. This should be always kept up to date by
				// the server, but depending on the implementation that might be different.
				self.last_cli_list = clients;
			},
			Packet::RequestGame(from) => println!("Incoming game request from: {}", from)
		}

		true
	}

	/// Shut the NetHandler down.
	pub fn shutdown(self) {
		drop(self.remote);

		if let Some(h) = self.rcv_handle {
			h.join().unwrap();
		}
	}

	/// Send a request to the server to send back the client list.
	/// This action does not wait until the list has been returned. The packet
	/// that is sent back will be processed through the normal channel.
	/// Returns true if the request has been sent successfully.
	pub fn request_client_list(&self) -> bool {
		let p = Packet::RequestClientList;
		self.remote.write_packet(&p)
	}

	/// Send a request to the player with the string. Returns true, if the Request could be made,
	/// not if the other client has accepted it.
	pub fn request_game(&self, to: String) -> bool {
		// Look if the client actually exists in the current client table
		let mut succ = false;
		for &(_, ref client) in &self.last_cli_list {
			if *client == to {
				succ = true;
			}
		}

		// Send the request to the server.
		if succ {
			let p = Packet::RequestGame(to);
			succ &= self.remote.write_packet(&p);
		}

		succ
	}
}
