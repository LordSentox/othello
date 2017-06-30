use std::sync::{Arc, Mutex};
use std::net::TcpStream;
use std::io::Read;

use srv::StatusTable;
use packets::*;

pub struct Player {
	name: Option<String>,
	status_table: Arc<Mutex<StatusTable>>,
	stream: TcpStream
}

impl Player {
	pub fn new(name: Option<String>, status_table: Arc<Mutex<StatusTable>>, stream: TcpStream) -> Player {
		Player {
			name: name,
			status_table: status_table,
			stream: stream
		}
	}

	// Handle the player input and stuff.
	pub fn run(&mut self) {
		loop {
			// Read all incoming packets and handle them accordingly.
			let mut id = vec![0; 1];
			let len = match self.stream.read(&mut id) {
				Ok(0) => {
					// The stream has been closed.
					println!("Connection has been closed by the client.");

					let mut lock = self.status_table.lock().unwrap();
					lock.remove_player(self.stream.peer_addr().unwrap());
					break;
				},
				Ok(len) => len,
				Err(err) => {
					println!("Error reading packet id: {}", err);
					continue;
				}
			};

			// Handle the packet according to its id.
			match id[0] {
				0 => {
					let mut packet = PChangeNameRequest{name: "unknown".to_string()};
					packet.read_from_stream(&mut self.stream);

					let mut lock = self.status_table.lock().unwrap();
					lock.name_player(&packet.name, self.stream.peer_addr().unwrap());
					println!("Renamed player to: {}", &packet.name);
				},
				_ => println!("Warning. ID [{}] from [{}] is invalid.", id[0], self.stream.peer_addr().unwrap())
			}

			println!("Packet: [{}] : [{}]", self.stream.peer_addr().unwrap(), id[0]);


			assert_eq!(len, 1);
		}
	}
}
