use std::any::Any;
use std::io;
use std::sync::{Arc, Mutex};
use game::Game;
use nethandler::*;
use packets::Packet;

pub struct RequestGameSequence {
	remote_name: String,
	local_ok: bool,
	remote_ok: bool,
	status: Status
}

impl RequestGameSequence {
	/// Start a request from the local player to the player with the provided name.
	/// game is a pointer to the game which might be started. It must be None initially.
	pub fn local_request(to: &str, handler: &NetHandler, game: Arc<Mutex<Option<Game>>>) -> Option<RequestGameSequence> {
		// Check if the game has already been started.
		if game.lock().unwrap().is_some() {
			println!("A game is already running. The request could not be made.");
			return None;
		}

		// Look if the client actually exists in the current client table
		let mut succ = false;
		for &(_, ref client) in handler.clients() {
			if *client == to {
				succ = true;
			}
		}

		if !succ {
			println!("The player you requested is not currently on the server.");
			return None;
		}

		// Send the request to the server.
		let p = Packet::RequestGame(to.to_string());
		if handler.write_packet(&p) {
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

	// Handle a remote request. This is at the Moment simply a blocking function, if there is no
	// game running, that is prompting the user on the spot if he wants to start a new game or not.
	// Returns true, if a new Game has been started, false otherwise.
	pub fn remote_request(remote_name: &str, handler: &NetHandler, game: Arc<Mutex<Option<Game>>>) -> bool {
		// Check if a game is already running. In that case the user will not be asked to choose,
		// but the request will be immediately blocked.
		if game.lock().unwrap().is_some() {
			println!("Received request from {}, but a game is already running.", remote_name);
			return false;
		}

		// Check if there is already a request from the client that is being
		// handled.
		for ref sequence in handler.sequences() {
			if let Some(req) = sequence.as_any().downcast_ref::<RequestGameSequence>() {
				if req.remote_name == remote_name {
					println!("Received request from {}, but a request is already noted.", remote_name);
					return false;
				}
			}
		}

		// Ask the player if he wants to accept the request, and read input.
		println!("Incoming game request from {}. Do you want to accept? [y/n]", remote_name);
		let mut accept = false;
		loop {
			let mut answer = String::new();
			match io::stdin().read_line(&mut answer) {
				Ok(_) => {},
				Err(err) => println!("Could not read line. Please try again.")
			}

			if answer.to_lowercase() == "y" {
				accept = true;
				break;
			}
			else if answer.to_lowercase() == "n" {
				accept = false;
				break;
			}
			else {
				println!("Invalid answer. Please only use y or n");
			}
		}

		// Send the response
		let packet = Packet::RequestGameResponse(handler.login_name(), accept);
		handler.write_packet(&packet);

		if !accept {
			println!("Denying game request.");
			return false;
		}

		// Start the actual game that will now be played.
		*game.lock().unwrap() = Some(Game::new());
		true
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

	fn on_success(&mut self) {
		unimplemented!();
	}
}
