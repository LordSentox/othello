extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

pub mod srv;
use srv::{Player, StatusTable};

pub mod packets;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
	let args: Vec<String> = std::env::args().collect();
	if args.len() < 2 {
		panic!("Could not establish server. Please specify a port to listen to.");
	}

	let port: u16 = args[1].parse().expect("Input formatted incorrectly. Could not read port.");

	let status_table: Arc<Mutex<StatusTable>> = Arc::new(Mutex::new(StatusTable::new()));

	let listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))).expect("Could not start listening. Shutting down.");
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				// Push the information of the new, momentarily unnamed player
				// into the status table.
				{	let mut lock = status_table.lock().unwrap();
					lock.add_player(None, stream.peer_addr().expect("Could not identify client address. Also, servers shouldn't panic, so please contact the lazy programmer."));
				}

				println!("A new client has connected: {}", stream.peer_addr().unwrap());

				// Move the Player to a new thread and run it on the stream.
				let status_table_clone = status_table.clone();
				thread::spawn(move || {
					let mut player = Player::new(None, status_table_clone, stream);

					player.run();
				});
			}
			Err(err) => println!("Client tried to connect, but an error occured. {}", err)
		}
	}
}
