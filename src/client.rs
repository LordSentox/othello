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

fn print_help() {
	println!("help -- show this message");
	println!("start -- Start a local game.");
	println!("connect <address> (<login_name>) -- Connect to the specified server.");
	println!("challenge <name/id> -- Challenge the client with the provided name or id to a Duel or accept a request by them.");
	println!("deny <name/id> -- Deny a game from the client, if the client had requested one.");
	println!("exit -- End the program.");
}

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
				print_help();
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

	let mut nethandler: Option<Arc<NetHandler>> = None;
	let mut client_list: Vec<(ClientId, String)> = Vec::new();
	let packets = Arc::new(Mutex::new(VecDeque::new()));

	let cmd_rcv = process_input();

	// All the games the client is currently engaged in.
	let mut games: Vec<Box<Game>> = Vec::new();
	loop {
		// If the client is currently not running any games, the thread will block
		// and wait for the next command. Otherwise this is obviously not possible,
		// so the input is non-blocking.
		// XXX: This will just return None, if the Sender has hung up at the moment.
		let cmd = if games.is_empty() {
			match cmd_rcv.recv_timeout(Duration::from_millis(200)) {
				Ok(cmd) => Some(cmd),
				Err(err) => None
			}
		}
		else {
			match cmd_rcv.try_recv() {
				Ok(cmd) => Some(cmd),
				Err(err) => None
			}
		};

		// Handle the received command, if any.
		if let Some(cmd) = cmd {
			let parts: Vec<&str> = cmd.split_whitespace().collect();
			if let Some(raw) = parts.get(0) {
				match raw {
					&"connect" => {
						if parts.len() != 2 {
							println!("Wrong number of arguments.");
							print_help();
						}
						if parts.len() == 2 {
							let login_name = match &CONFIG.network.login_name {
								&Some(ref name) => name.clone(),
								&None => {
									println!("Login name could not be read from configuration file. Please try again, but provide a name in the command.");

								}
							};

							// Create the connection to the server.
							nethandler = match NetHandler::connect(&parts[1], &login_name) {
								Ok(n) => {
									n.subscribe(Arc::downgrade(&packets));
									Some(n)
								},
								Err(err) => {
									println!("Could not connect to server {:?}", err);
									None
								}
							};
						}
					},
					&"start" => {
						if parts.len() != 1 {
							println!("Wrong number of arguments.");
							print_help();
						}
						else {
							games.push(Box::new(OfflineGame::new()));
						}
					}
					&"challenge" => {
						if parts.len() != 2 {
							println!("Wrong number of arguments.");
							print_help();
						}
						else {
							// Try find the client with the corresponding name.
							let mut found = false;
							for &(ref id, ref name) in &client_list {
								if name.to_lowercase() == parts[1].to_lowercase() {
									nethandler.as_ref().unwrap().send(&Packet::RequestGame(*id));
									println!("Requested game from client [{}]: {}", id, name);
									found = true;
									break;
								}
							}

							// Try parse the argument as id and send the request to the client.
							if found {}
							else if let Ok(requestee) = parts[1].parse::<ClientId>() {
								for &(ref id, ref name) in &client_list {
									if *id == requestee {
										nethandler.as_ref().unwrap().send(&Packet::RequestGame(*id));
										println!("Requested game from client [{}]: {}", id, name);
										found = true;
										break;
									}
								}
							}

							if !found {
								println!("Could not find client '{}'", parts[1]);
							}
						}
					},
					&"deny" => {
						if parts.len() != 2 {
							println!("Wrong number of arguments.");
							print_help();
						}
						else {
							// Try find the client with the corresponding name.
							for &(ref id, ref name) in &client_list {
								if name.to_lowercase() == parts[1].to_lowercase() {
									nethandler.as_ref().unwrap().send(&Packet::DenyGame(*id));
									println!("Denying game from client [{}]: {}", id, name);
								}
							}

							// Try parse the argument as id and send the request to the client.
							if let Ok(requestee) = parts[1].parse::<ClientId>() {
								for &(ref id, ref name) in &client_list {
									if *id == requestee {
										nethandler.as_ref().unwrap().send(&Packet::DenyGame(*id));
										println!("Denying game from client [{}]: {}", id, name);
									}
								}
							}
							else {
								println!("Could not find client '{}'", parts[1]);
							}
						}
					},
					&"exit" => {
						if parts.len() != 1 {
							println!("Wrong number of arguments.");
							print_help();
						}
						else {
							break;
						}
					}
					_ => {
						println!("Unknown command.");
						print_help();
					}
				}
			}
			else {
				println!("No command entered.");
			}
		}

		// Handle incoming packets.
		loop {
			let packet = match packets.lock().unwrap().pop_front() {
				Some(p) => p,
				None => break
			};

			let mut packet_handled = false;
			for ref mut game in &mut games {
				if game.handle_packet(&packet) {
					packet_handled = true;
					break;
				}
			}

			if !packet_handled {
				// The packet is of a different kind.
				match packet {
					Packet::ClientList(clients) => client_list = clients,
					Packet::RequestGame(client) => println!("Client [{}] has requested a game. Use challenge to accept the request.", client),
					Packet::Message(client, message) => println!("[{}]: {}", client, message),
					Packet::StartGame(opponent, piece) => games.push(Box::new(OnlineGame::new(nethandler.as_ref().unwrap().clone(), piece, opponent))),
					p => println!("{:?} was not handled.", p)
				}
			}
		}

		for ref mut game in &mut games {
			game.handle_events();
			game.draw();
		}

		games.retain(|ref game| { game.running() });
	}
}
