extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate toml;
extern crate lazy_static;

pub mod board;
pub mod packets;
pub mod remote;
pub mod srv;

use srv::{NetHandler, Master};

fn main() {
	let args: Vec<String> = std::env::args().collect();
	if args.len() < 2 {
		panic!("Could not establish server. Please specify a port to listen to.");
	}

	let port: u16 = args[1].parse().expect("Input formatted incorrectly. Could not read port.");

	let nethandler = NetHandler::start_listen(port).expect("Could not start NetHandler.");

	let master = Master::new(nethandler.clone());

	loop {
		master.handle_packets();
	}
}
