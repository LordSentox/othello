//! Handles the console input, notably processes commands entered by the user and relays them
//! properly to the correct handler for them.
use std::sync::mpsc::{self, Sender, Receiver};
use std::io::{self, BufRead};
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

fn process_input(sender: Sender<Vec<String>>) {
	let lines = io::stdin().lock().lines();
	for line in lines {
		let line = match line {
			Ok(line) => line,
			Err(err) => {
				println!("Error reading command. {:?}", err);
				continue;
			}
		};

		let command: Vec<String> = line.trim_right_matches("\n").split_whitespace().map(|ref part| { part.to_string() }).collect();

		sender.send(command).unwrap();
	}

	thread::sleep(Duration::from_millis(50));
}

pub struct Console {
	/// Receives the commands entered on the managing input thread, so that they can
	/// be handled at the proper time.
	receiver: Receiver<Vec<String>>,
	/// Is the console still running? Used to terminate the input thread.
	running: Arc<AtomicBool>,
	/// Join Handle to ensure the Console never outlives the input thread.
	handle: JoinHandle<()>,
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
		let handle = thread::spawn(move || {
			while running.load(Ordering::Relaxed) {
				process_input(sender);
			}
		});

		Console {
			receiver: receiver,
			running: running,
			handle: handle,
			exit_command: exit_command
		}
	}
}

impl Drop for Console {
	fn drop(&mut self) {
		// Ask the input thread to terminate itself and wait until it is finished.
		self.running.store(false, Ordering::Relaxed);
	}
}
