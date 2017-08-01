use std::any::Any;
use nethandler::*;
use packets::Packet;

pub struct RequestGameSequence {
	remote_name: String,
	local_ok: bool,
	remote_ok: bool,
	status: Status
}

impl RequestGameSequence {
	pub fn local_request(to: &str, handler: &NetHandler) -> Option<RequestGameSequence> {
		// Look if the client actually exists in the current client table
		let mut succ = false;
		for &(_, ref client) in handler.clients() {
			if *client == to {
				succ = true;
			}
		}

		// Send the request to the server.
		if succ {
			let p = Packet::RequestGame(to.to_string());
			succ &= handler.write_packet(&p);
		}

		if succ {
			// The request has been sent. Now it's time to wait for the response.
			Some(RequestGameSequence {
				remote_name: to.to_string(),
				local_ok: true,
				remote_ok: false,
				status: Status::Running
			})
		}
		else {
			None
		}
	}

	pub fn remote_request(remote_name: &str, handler: &NetHandler) -> Option<RequestGameSequence> {
		// Check if there is already a request from the client that is being
		// handled.
		for ref sequence in handler.sequences() {
			if let Some(req) = sequence.as_any().downcast_ref::<RequestGameSequence>() {
				if req.remote_name == remote_name {
					println!("Received request from {}, but a request is already noted.", remote_name);
					return None;
				}
			}
		}

		unimplemented!();
	}
}

impl PacketSequence for RequestGameSequence {
	fn status(&self) -> Status {
		self.status
	}

	fn as_any(&self) -> &Any {
		self
	}

	fn on_packet(&mut self, packet: &Packet) -> bool {
		unimplemented!();
	}

	fn on_success(&self, handler: &mut NetHandler) {
		unimplemented!();
	}
}
