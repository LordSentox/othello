extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate toml;

#[macro_use]
pub mod packets;
pub mod srv;

use srv::NetHandler;

fn main() {
	let args: Vec<String> = std::env::args().collect();
	if args.len() < 2 {
		panic!("Could not establish server. Please specify a port to listen to.");
	}

	let port: u16 = args[1].parse().expect("Input formatted incorrectly. Could not read port.");

	let _ = NetHandler::new(port).unwrap();

	loop {}
}
