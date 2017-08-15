extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate toml;
#[macro_use]
extern crate lazy_static;

pub mod board;
pub mod packets;
pub mod remote;
pub mod srv;

use std::thread;
use std::time::Duration;

use srv::{CONFIG, NetHandler, GameHandler, Master};

fn main() {
	let nethandler = NetHandler::start_listen(CONFIG.port).expect("Could not start NetHandler.");

	let mut master = Master::new(nethandler.clone());
	let mut gamehandler = GameHandler::new(nethandler.clone());

	loop {
		master.handle_packets();
		gamehandler.handle_packets();

		// TODO: The thread really should just be unparked whenever a packet comes
		// in, to waste neither time, nor resources, but this will do for now to
		// stop the excessive resource grabbing.
		thread::sleep(Duration::from_millis(500));
	}
}
