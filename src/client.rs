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
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::Duration;

use cli::*;
use packets::*;



/// Reads the user input in a new thread. If it cannot be interpreted on the spot,
/// it will be sent to the receiver.
fn process_input_old() -> Receiver<String> {
	let (sender, receiver) = mpsc::channel();

	thread::spawn(move || {
		loop {
			let mut cmd = String::new();
			if io::stdin().read_line(&mut cmd).is_err() {
				println!("Could not read command. Please try again.");
			}

			let cmd = cmd.trim_right_matches("\n").to_string();

			if cmd == "help" {
				// print_help();
			}
			else {
				if sender.send(cmd).is_err() {
					println!("Receiver could not receive command. Exiting input thread.");
					break;
				}
			}
		}
	});

	receiver
}

fn main() {
	println!("Welcome to othello.");

	// Package the parts needed in the console into a context, so they can be passed more easily.
	let mut ctx = Context {
		nethandler: None,
		client_list: Vec::new(),
		games: Vec::new(),
		packets: Arc::new(Mutex::new(VecDeque::new()))
	};

	let console = Console::new("exit");
	while console.running() {
		// If the client is currently not running any games, the thread will block
		// and wait for the next command. Otherwise this is obviously not possible,
		// so the input is non-blocking.
		// XXX: This will just return None, if the Sender has hung up at the moment.

		// Handle the received commands, if any.
		let blocking = ctx.games.is_empty();
		console.handle_commands(&mut ctx, blocking);

		// Handle incoming packets.
		loop {
			let packet = match ctx.packets.lock().unwrap().pop_front() {
				Some(p) => p,
				None => break
			};

			let mut packet_handled = false;
			for ref mut game in &mut ctx.games {
				if game.handle_packet(&packet) {
					packet_handled = true;
					break;
				}
			}

			if !packet_handled {
				// The packet is of a different kind.
				match packet {
					Packet::ClientList(clients) => ctx.client_list = clients,
					Packet::RequestGame(client) => println!("Client [{}] has requested a game. Use challenge to accept the request.", client),
					Packet::Message(client, message) => println!("[{}]: {}", client, message),
					Packet::StartGame(opponent, piece) => ctx.games.push(Box::new(OnlineGame::new(ctx.nethandler.as_ref().unwrap().clone(), piece, opponent))),
					p => println!("{:?} was not handled.", p)
				}
			}
		}

		for ref mut game in &mut ctx.games {
			game.handle_events();
			game.draw();
		}

		ctx.games.retain(|ref game| { game.running() });
	}
}
