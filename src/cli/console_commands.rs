//! Handles the console input, notably processes commands entered by the user and relays them
//! properly to the correct handler for them.
use std::sync::mpsc::{self, Sender, Receiver};
use std::io;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::collections::VecDeque;

use cli::{Game, OfflineGame, NetHandler, CONFIG};
use packets::*;

pub struct Context {
	pub nethandler: Option<Arc<NetHandler>>,
	pub client_list: Vec<(ClientId, String)>,
	pub games: Vec<Box<Game>>,
	pub packets: Arc<Mutex<VecDeque<Packet>>>
}

fn print_help() {
	println!("help -- show this message");
	println!("start -- Start a local game.");
	println!("connect <address> (<login_name>) -- Connect to the specified server.");
	println!("challenge <name/id> -- Challenge the client with the provided name or id to a Duel or accept a request by them.");
	println!("deny <name/id> -- Deny a game from the client, if the client had requested one.");
	println!("exit -- End the program.");
}

fn find_name_or_id<'a>(client_list: &'a Vec<(ClientId, String)>, to_find: &String) -> Option<(ClientId, &'a String)> {
	// Try find the client with the corresponding name.
	for &(ref id, ref name) in client_list {
		if name.to_lowercase() == to_find.to_lowercase() {
			return Some((*id, name));
		}
	}

	// If the client could not be found by name, maybe it was entered as an id. In that case,
	// try to parse the id from the argument.
	if let Ok(requestee) = to_find.parse::<ClientId>() {
		for &(ref id, ref name) in client_list {
			if *id == requestee {
				return Some((*id, name));
			}
		}
	}

	None
}

mod cmd {
	use super::*;

	pub enum Error {
		WrongNumberOfArguments,
		NeedsConnection,
		PlayerNotFound,
		UnknownCommand(String)
	}

	pub fn connect(ctx: &mut Context, args: Vec<String>) -> Result<(), Error> {
		// To connect, we need at least the IP of the server.
		if args.len() < 1 {
			return Err(Error::WrongNumberOfArguments);
		}

		let login_name = if args.len() == 1 {
			match &CONFIG.network.login_name {
				&Some(ref name) => name,
				&None => {
					println!("Login name could not be read from configuration file. Please try again, but provide a name in the command.");
					return Ok(());
				}
			}
		}
		else if args.len() == 2 {
			if CONFIG.network.login_name.is_some() {
				println!("[WARNING] Overriding the login name configured in client.toml");
			}

			&args[1]
		}
		else {
			return Err(Error::WrongNumberOfArguments);
		};

		// If the client was previously connected, they cannot reconnect.
		// TODO: This is, because the Disconnects are not nicely handled at the moment. Later, the
		// client should obviously just be warned, that they are about to change the server, but
		// then everything should continue as it would normally.
		if let Some(ref net) = ctx.nethandler.as_ref() {
			println!("You are already connected to a server. Please disconnect first.");
			return Ok(());
		}

		// Create the connection to the server.
		ctx.nethandler = match NetHandler::connect(&args[0], &login_name) {
			Ok(n) => {
				n.subscribe(Arc::downgrade(&ctx.packets));
				Some(n)
			},
			Err(err) => {
				println!("Could not connect to server {:?}", err);
				return Ok(());
			}
		};

		println!("Connected.");
		Ok(())
	}

	pub fn start(ctx: &mut Context, args: Vec<String>) -> Result<(), Error> {
		if !args.is_empty() {
			return Err(Error::WrongNumberOfArguments);
		}

		ctx.games.push(Box::new(OfflineGame::new()));
		Ok(())
	}

	pub fn challenge(ctx: &mut Context, args: Vec<String>) -> Result<(), Error> {
		if ctx.nethandler.is_none() {
			return Err(Error::NeedsConnection);
		}
		if args.len() != 1 {
			return Err(Error::WrongNumberOfArguments);
		}

		// Either the user has entered an id, or a name. Try to find the corresponding client by
		// trying both.
		let client = find_name_or_id(&ctx.client_list, &args[0]);

		if client.is_none() {
			return Err(Error::PlayerNotFound);
		}
		let (id, name) = client.unwrap();

		ctx.nethandler.as_ref().unwrap().send(&Packet::RequestGame(id));
		println!("Requested game from client [{}]: {}", id, name);
		Ok(())
	}

	pub fn deny(ctx: &mut Context, args: Vec<String>) -> Result<(), Error> {
		if ctx.nethandler.is_none() {
			return Err(Error::NeedsConnection);
		}
		if args.len() != 1 {
			return Err(Error::WrongNumberOfArguments);
		}

		// Either the user has entered an id, or a name. Try to find the corresponding client by
		// trying both.
		let client = find_name_or_id(&ctx.client_list, &args[0]);

		if client.is_none() {
			return Err(Error::PlayerNotFound);
		}
		let (id, name) = client.unwrap();

		ctx.nethandler.as_ref().unwrap().send(&Packet::DenyGame(id));
		println!("Denied game from client [{}]: {}", id, name);
		Ok(())
	}
}

fn process_input_line(sender: &Sender<Vec<String>>) {
	let mut line = String::new();
	match io::stdin().read_line(&mut line) {
		Ok(_) => {},
		Err(err) => {
			println!("Error reading command. {:?}", err);
			return;
		}
	};

	let command: Vec<String> = line.trim_right_matches("\n").split_whitespace().map(|ref part| { part.to_string() }).collect();

	sender.send(command).unwrap();
}

pub struct Console {
	/// Receives the commands entered on the managing input thread, so that they can
	/// be handled at the proper time.
	receiver: Receiver<Vec<String>>,
	/// Is the console still running? Used to terminate the input thread.
	running: Arc<AtomicBool>,
	/// The command on which the Console will be terminated.
	exit_command: &'static str
}

impl Console {
	/// Starts receiving input from the user and returns the Console, which can then
	/// be used to handle the input in the same thread as it was called in.
	pub fn new(exit_command: &'static str) -> Console {
		let (sender, receiver) = mpsc::channel();
		let running = Arc::new(AtomicBool::new(true));

		// Start the input thread.
		let running_clone = running.clone();
		thread::spawn(move || {
			while running_clone.load(Ordering::Relaxed) {
				process_input_line(&sender);

				thread::sleep(Duration::from_millis(50));
			}
		});

		Console {
			receiver: receiver,
			running: running,
			exit_command: exit_command
		}
	}

	/// Handle the commands the user has entered. If blocking is set to true, the console will
	/// wait for commands for a specified amount of time, but not forever. If set to false, the
	/// console will only block to handle commands if there is anything available.
	pub fn handle_commands(&self, context: &mut Context, blocking: bool) {
		// This function may not be called when the Console is not running anymore. It's also
		// useless, since nothing can happen.
		assert!(self.running.load(Ordering::Relaxed));

		let mut cmd = if blocking {
			match self.receiver.recv_timeout(Duration::from_millis(200)) {
				Ok(cmd) => cmd,
				Err(err) => {
					// TODO: If the Error is not a timeout error, print it here.
					// println!("Error reading command. {:?}", err);
					return;
				}
			}
		}
		else {
			match self.receiver.try_recv() {
				Ok(cmd) => cmd,
				Err(err) => {
					// TODO: If the Error is not a timeout error, print it here.
					// println!("Error reading command. {:?}", err);
					return;
				}
			}
		};

		if cmd.is_empty() {
			println!("No command entered. Try help to get a list of available commands.");
			return;
		}

		// The raw command without any arguments.
		let raw = cmd.remove(0);
		let cmd = cmd;

		if raw == self.exit_command {
			self.running.store(false, Ordering::Relaxed);
			println!("Exiting..");
			return;
		}

		// Handle the command and save the Result of the operation.
		let res = match &raw.as_str() {
			&"connect" => cmd::connect(context, cmd),
			&"start" => cmd::start(context, cmd),
			&"challenge" => cmd::challenge(context, cmd),
			&"deny" => cmd::deny(context, cmd),
			&"help" => { print_help(); Ok(()) },
			c => Err(cmd::Error::UnknownCommand(c.to_string()))
		};

		match res {
			Ok(()) => {},
			Err(cmd::Error::WrongNumberOfArguments) => println!("Wrong number of arguments. See 'help' for usage information."),
			Err(cmd::Error::NeedsConnection) => println!("You need to be connected to a Server for this."),
			Err(cmd::Error::PlayerNotFound) => println!("Could not find player. Please make sure the id or name is valid."),
			Err(cmd::Error::UnknownCommand(c)) => println!("Unknown command '{}'. See 'help' for options.", c)
		}
	}

	/// Check if the Console is still running, or if it has either been cancelled already or received an exit command.
	pub fn running(&self) -> bool {
		self.running.load(Ordering::Relaxed)
	}
}

impl Drop for Console {
	fn drop(&mut self) {
		// Ask the input thread to terminate itself and wait until it is finished.
		self.running.store(false, Ordering::Relaxed);

		// XXX: This simply doesn't work, because Rust has no support for read timeouts on stdin
		// whatsoever. Putting the exit command directly into the read thread is also not an option,
		// since that would still mean I cannot exit the console from anywhere else. So, damn the
		// input thread.
		// self.handle.take().unwrap().join().expect("Could not join input thread properly.");
	}
}
