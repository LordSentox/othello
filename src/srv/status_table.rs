use std::sync::{Arc, Mutex};
use std::net::{SocketAddr, ToSocketAddrs};
use std::net::TcpStream;
use packets::Packet;
use srv::player::Player;
use std::boxed::Box;

pub struct StatusTable {
	// TODO: Create or find a structure that can handle the search for individual
	// indices better than a simple unsorted Vector with linear search.
	players: Vec<(Option<String>, TcpStream)>
}

impl StatusTable {
	pub fn new() -> StatusTable {
		StatusTable {
			players: Vec::new()
		}
	}

	pub fn add_player(&mut self, name: Option<String>, stream: TcpStream) -> bool {
		// Add the player, if the name has not already been taken.
		match name {
			None => {
				self.players.push((None, stream));
				true
			},
			Some(name) => {
				if let Some(_) = self.find_player(&name) {
					false
				}
				else {
					self.players.push((Some(name), stream));
					true
				}
			}
		}
	}

	pub fn remove_player(&mut self, player: &Player) -> bool {
		// Remember the length before, so we can check if a player has been removed.
		let len = self.players.len();

		self.players.retain(|&(_, ref comp_stream)| player.stream().peer_addr().unwrap() != comp_stream.peer_addr().unwrap());

		len > self.players.len()
	}

	pub fn find_player(&self, name: &String) -> Option<TcpStream> {
		for &(ref comp_name, ref stream) in &self.players {
			if let &Some(ref comp_name) = comp_name {
				if name.clone() == *comp_name {
					return Some(stream.try_clone().expect("Could not clone stream"));
				}
			}
		}

		None
	}

	pub fn name_player(&mut self, new_name: &String, stream: &TcpStream) -> bool {
		// A player with the same name cannot be added twice
		if let Some(_) = self.find_player(&new_name) {
			return false;
		};

		for &mut (ref mut name, ref mut comp_stream) in &mut self.players {
			if stream.peer_addr().unwrap() == comp_stream.peer_addr().unwrap() {
				*name = Some(new_name.clone());
				return true;
			}
		};

		false
	}

	pub fn broadcast_packet_named<P: Packet>(&mut self, packet: &P) -> bool {
		let mut one_failed = false;
		for &mut (ref mut name, ref mut stream) in &mut self.players {
			if let &mut Some(ref mut name) = name {
				one_failed |= !packet.write_to_stream(&mut stream);
			}
		}

		one_failed
	}

	pub fn broadcast_packet<P: Packet>(&mut self, packet: &P) -> bool {
		let mut one_failed = false;
		for &mut (_, ref mut stream) in &mut self.players {
			one_failed |= !packet.write_to_stream(&mut stream);
		}

		one_failed
	}

	pub fn players(&self) -> &Vec<(Option<String>, TcpStream)> {
		&self.players
	}
}
