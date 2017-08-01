use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use std::sync::{Arc, Weak};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::thread;
use std::thread::JoinHandle;
use login_sequence::LoginSequence;
use request_game_sequence::RequestGameSequence;
use std::any::Any;
use remote::*;
use std::io::Error as IOError;
use packets::*;

#[derive(Debug)]
pub enum Error {
	IO(IOError),
	NameChangeFailed,
	Receive(PacketReadError)
}

#[derive(Copy, Clone)]
pub enum Status {
	Running,
	Finished(bool)
}

pub trait PacketSequence {
	/// The status of the packet sequence.
	fn status(&self) -> Status;

	/// Get an Any type from the Sequence, to dynamically check which type of
	/// sequence is referenced.
	fn as_any(&self) -> &Any;

	/// When a packet comes in, it is checked, if this PacketSequence has something to do
	/// with it. Returns true, if the packet is captured. The packet will not be processed
	/// any further.
	fn on_packet(&mut self, packet: &Packet) -> bool;

	/// When the status returns Finished(true), this function is called.
	fn on_success(&self, handler: &mut NetHandler) {}

	/// When the status returns Finished(false), this function is called.
	fn on_failure(&self, handler: &mut NetHandler) {}
}

pub struct NetHandler {
	remote: Arc<Remote>,
	rcv_handle: Option<JoinHandle<()>>,
	p_rcv: Option<Receiver<Packet>>,
	last_cli_list: Vec<(ClientId, String)>,
	sequences: Vec<Box<PacketSequence>>
}

impl NetHandler {
	/// Attempts to connect to a server on the given address with the provided
	/// login name.
	/// Returns a NetHandler on success. The NetHandler will however not start
	/// receiving packets until explicitly asked.
	pub fn new<A>(addrs: A, name: &str) -> Result<NetHandler, Error> where A: ToSocketAddrs {
		// Try to connect to the server.
		let stream = match TcpStream::connect(addrs) {
			Ok(stream) => stream,
			Err(err) => return Err(Error::IO(err))
		};

		let remote = match Remote::new(stream) {
			Ok(remote) => Arc::new(remote),
			Err(err) => { return Err(Error::IO(err)); }
		};

		// Start the login sequence and wait until it is finished.
		let mut login = LoginSequence::start(remote.clone(), name).unwrap();

		while let Status::Running = login.status() {
			match remote.read_packet() {
				Ok(p) => { login.on_packet(&p); },
				Err(err) => return Err(Error::Receive(err))
			}
		}

		if let Status::Finished(false) = login.status() {
			return Err(Error::NameChangeFailed);
		}

		Ok(NetHandler {
			remote: remote,
			rcv_handle: None,
			p_rcv: None,
			last_cli_list: Vec::new(),
			sequences: Vec::new()
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
		let p = {
			let p_rcv: &Receiver<Packet> = self.p_rcv.as_ref().unwrap();

			match p_rcv.try_recv() {
				Ok(p) => p,
				Err(err) => { println!("Error handling packet. {}", err); return false; }
			}
		};

	 	// Currently running sequences might be interested in the packet.
		// The sequences are iterated in reverse order, so removing an
		// element does not make the algorithm skip anything.
		// for i in (0..self.sequences.len()).rev() {
		// 	// Handle the packet and stop continuing in case it has been captured.
		// 	let captured = self.sequences[i].on_packet(&p);
		//
		// 	let mut cur_seq = self.sequences.get_mut(i).expect("Index out of bounds.");
		// 	match cur_seq.status() {
		// 		Status::Finished(true) => {
		// 			cur_seq.on_success(self);
		// 			self.sequences.swap_remove(i);
		// 		}
		// 		Status::Finished(false) => {
		// 			self.sequences[i].on_failure(self);
		// 			self.sequences.swap_remove(i);
		// 		}
		// 		Status::Running => {}
		// 	}
		//
		// 	// If the packet has been captured, the rest of the function can be skipped.
		// 	if captured {
		// 		return false;
		// 	}
		// }

		// Handle packets that are not part of a specific sequence.
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
			Packet::RequestGame(from) => println!("Incoming game request from: {}", from),
			_ => { unimplemented!(); }
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

	/// Write a custom packet to the server.
	pub fn write_packet(&self, p: &Packet) -> bool {
		self.remote.write_packet(&p)
	}

	/// Send a request to the server to send back the client list.
	/// This action does not wait until the list has been returned. The packet
	/// that is sent back will be processed through the normal channel.
	/// Returns true if the request has been sent successfully.
	pub fn request_client_list(&self) -> bool {
		let p = Packet::RequestClientList;
		self.remote.write_packet(&p)
	}

	/// The last client list that has been received from the server. This should always
	/// be updated by the server, but generally it should be taken with a grain of salt.
	pub fn clients(&self) -> &Vec<(ClientId, String)> {
		&self.last_cli_list
	}

	/// All PacketSequences this handler is currently handling.
	pub fn sequences(&self) -> &Vec<Box<PacketSequence>> {
		&self.sequences
	}

	/// Send a request to the player with the string. Returns true, if the Request could be made,
	/// not if the other client has accepted it.
	pub fn request_game(&mut self, to: &str) -> bool {
		match RequestGameSequence::local_request(to, &self) {
			Some(request) => {
				self.sequences.push(Box::new(request));
				true
			}
			None => {
				false
			}
		}
	}
}
