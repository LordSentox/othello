use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::sync::{Arc, Weak};
use std::thread;
use std::net::{TcpStream, ToSocketAddrs};
use remote::Remote;
use packets::*;

pub struct Server {
	remote: Weak<Remote>,
	packet_receiver: Receiver<Packet>
}

impl Server {
	/// Try to connect to the server at the given address and attemts to set the
	/// name of the client to the given string. If it is successful,
	/// the function returns the server. If not, it prints the error and returns None.
	// TODO: Yes, I know it would be better to return a Result.. ksh!
	pub fn new<A: ToSocketAddrs>(addr: A, name: &String) -> Option<Server> {
		let stream = match TcpStream::connect(addr) {
			Ok(stream) => stream,
			Err(err) => { println!("Could not connect to server: {}", err); return None;}
		};

		let remote = match Remote::new(stream) {
			Ok(remote) => remote,
			Err(err) => { println!("Could not connect to server: {}", err); return None;}
		};

		// The connection has been established. Now we will change the name of
		// the client on the server, so that other clients can identify us easier.
		let p = Packet::ChangeNameRequest(name.clone());
		remote.write_packet(&p);

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
					return None;
				}
				Ok(p) => println!("Received unexpected packet from server: {:?} .. ignoring", p),
				Err(err) => {
					println!("Error occured while sending name to server: {:?}", err);
					return None;
				}
			}
		}


		let (sender, receiver) = mpsc::channel();

		// The remote gets cloned to get the weak reference later. It will be
		// owned by the packet receiving thread from here on.
		let remote = Arc::new(remote);
		Server::start_receiving(remote.clone(), sender);

		Some(Server {
			remote: Arc::downgrade(&remote),
			packet_receiver: receiver
		})
	}

	/// Start receiving packets from the server. This starts a new thread.
	/// When a packet is received it is pushed into the queue of the provided sender.
	fn start_receiving(remote: Arc<Remote>, sender: Sender<Packet>) {
		thread::spawn(move || {
			loop {
				match remote.read_packet() {
					Ok(p) => sender.send(p).unwrap(),
					Err(PacketReadError::Closed) => {
						println!("The server has closed the connection.");
						break;
					},
					Err(err) => println!("Error receiving packet. {:?}", err)
				}
			}
		});
	}

	/// Handle all packets that have been received from the server since the
	/// last time this function has been called.
	pub fn handle_packets(&self) {
		for p in self.packet_receiver.try_iter() {
			
		}

		unimplemented!();
	}

	/// Send a request to the server to send back the client list.
	/// This action does not wait until the list has been returned. The packet
	/// that is sent back will be processed through the normal channel.
	/// Returns true if the request has been sent successfully.
	pub fn request_client_list(&self) -> bool {
		let p = Packet::RequestClientList;
		self.remote.upgrade().unwrap().write_packet(&p)
	}
}
