extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
#[macro_use]
extern crate lazy_static;
extern crate sfml;
extern crate toml;

pub mod board;
pub mod cli;
pub mod packets;
pub mod remote;
pub mod score;

use std::io::{self, Write};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use board::{Board, Piece};
use score::{Score};

use sfml::window::{ContextSettings, VideoMode, Event, style};
use sfml::window::mouse::Button;
use sfml::graphics::{Color, Rect, RenderTarget, RenderWindow};

use cli::*;
use packets::Packet;

/// Reads the user input in a new thread. If it cannot be interpreted on the spot,
/// it will be sent to the receiver.
fn process_input() -> Receiver<String> {
	let (sender, receiver) = mpsc::channel();

	thread::spawn(move || {
		loop {
			let mut cmd = String::new();
			if io::stdin().read_line(&mut cmd).is_err() {
				println!("Could not read command. Please try again.");
			}

			let cmd = cmd.trim_right_matches("\n").to_string();

			if cmd == "help" {
				println!("help -- show this message.");
			}
			else {
				sender.send(cmd);
			}
		}
	});

	receiver
}

fn main() {
	println!("Welcome to othello.");

	let login_name = match &CONFIG.network.login_name {
		&Some(ref name) => name.clone(),
		&None => {
			print!("Login: ");
			if io::stdout().flush().is_err() { println!(""); }
			let mut login_name = String::new();
			io::stdin().read_line(&mut login_name).expect("Could not read login name. Aborting. {}");

			login_name.trim_right_matches("\n").to_string()
		}
	};

	// Create the connection to the server.
	// TODO: Handle the errors a little bit more gracefully.
	let nethandler = NetHandler::connect((CONFIG.network.server_ip.as_str(), CONFIG.network.server_port), &login_name).expect("Could not connect to the server.");
	let packets = Arc::new(Mutex::new(VecDeque::new()));
	nethandler.subscribe(Arc::downgrade(&packets));

	let cmd_rcv = process_input();

	// All the games the client is currently engaged in.
	let mut games: Vec<Game> = Vec::new();
	loop {
		// If the client is currently not running any games, the thread will block
		// and wait for the next command. Otherwise this is obviously not possible,
		// so the input is non-blocking.
		// TODO
	}
}
