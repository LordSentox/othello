use std::sync::{Arc, Mutex};
use std::net::{SocketAddr, ToSocketAddrs};

pub struct StatusTable {
	// TODO: Create or find a structure that can handle the search for individual
	// indices better than a simple unsorted Vector with linear search.
	players: Vec<(Option<String>, SocketAddr)>
}

impl StatusTable {
	pub fn new() -> StatusTable {
		StatusTable {
			players: Vec::new()
		}
	}

	pub fn add_player(&mut self, name: Option<String>, addr: SocketAddr) -> bool {
		match name {
			None => {
				self.players.push((None, addr));
				true
			},
			Some(name) => {
				if let Some(_) = self.find_player(&name) {
					false
				}
				else {
					self.players.push((Some(name), addr));
					true
				}
			}
		}
	}

	pub fn remove_player(&mut self, addr: SocketAddr) -> bool {
		// Remember the length before, so we can check if a player has been removed.
		let len = self.players.len();

		self.players.retain(|&(_, comp_addr)| comp_addr != addr);

		len > self.players.len()
	}

	pub fn find_player(&self, name: &String) -> Option<SocketAddr> {
		for &(ref comp_name, ref addr) in &self.players {
			if let &Some(ref comp_name) = comp_name {
				if name.clone() == *comp_name {
					return Some(addr.clone());
				}
			}
		}

		None
	}

	pub fn name_player(&mut self, new_name: &String, addr: SocketAddr) -> bool {
		// A player with the same name cannot be added twice
		if let Some(_) = self.find_player(&new_name) {
			return false;
		};

		for &mut (ref mut name, ref mut comp_addr) in &mut self.players {
			if addr == *comp_addr {
				*name = Some(new_name.clone());
				return true;
			}
		};

		false
	}
}
