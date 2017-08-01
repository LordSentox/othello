use nethandler::*;

use std::any::Any;
use std::sync::Arc;
use remote::Remote;
use packets::*;

pub struct LoginSequence {
	status: Status
}

impl LoginSequence {
	/// Start the login sequence. Sends the request to the server.
	pub fn start(remote: Arc<Remote>, name: &str) -> Option<LoginSequence> {
		let p = Packet::ChangeNameRequest(name.to_string());
		if !remote.write_packet(&p) {
			println!("Login failed. Could not write to server.");
			return None;
		}

		Some(LoginSequence {
			status: Status::Running
		})
	}
}

impl PacketSequence for LoginSequence {
	fn status(&self) -> Status {
		self.status
	}

	fn as_any(&self) -> &Any {
		self
	}

	fn on_packet(&mut self, packet: &Packet) -> bool {
		// Nothing to do, if the login sequence has already been finished.
		if let Status::Finished(_) = self.status {
			return false;
		}

		match packet {
			&Packet::ChangeNameResponse(true) => {
				println!("Name has been set on the server.");
				self.status = Status::Finished(true);
				true
			},
			&Packet::ChangeNameResponse(false) => {
				println!("Could not login. Name has been rejected by the server.");
				self.status = Status::Finished(false);
				true
			},
			_ => { false } // Packet has nothing to do with the login sequence.
		}
	}
}
