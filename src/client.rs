extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate sfml;
extern crate toml;

pub mod board;
pub mod cli;
pub mod packets;
pub mod remote;
pub mod score;

use board::{Board, Piece};
use score::{Score};

use sfml::window::{ContextSettings, VideoMode, Event, style};
use sfml::window::mouse::Button;
use sfml::graphics::{Color, RenderTarget, RenderWindow};

use cli::*;

fn main() {
	// Connect to a server
	println!("Welcome to othello.");
	let args: Vec<String> = std::env::args().collect();
	if args.len() < 2 {
		panic!("Could not start client. Please provide the IP of the server you want to connect to.");
	}

	if args.len() < 3 {
		panic!("Could not start client. Please provide a name to identify yourself on the server.");
	}

	// Create the connection to the server. Later, this could be read from a
	// config file, with a little more galant error checking as well.
	let mut nethandler = NetHandler::new(&args[1], &args[2]).unwrap();
	nethandler.start_receiving();

	// Create the window of the application
	let mut window = RenderWindow::new(VideoMode::new(512, 532, 32), "SFML Othello", style::CLOSE, &ContextSettings::default()).unwrap();
	window.set_framerate_limit(30);

	// Create a test board and print its contents.
	let mut board = Board::new();
	board.print();

	// Create the Score Bar
	let mut score = Score::new(&board);

	// The player to set the first stone is black.
	let mut next_piece = Piece::BLACK;

	loop {
		for event in window.events() {
			if let Event::Closed = event {
				return;
			}
			else if let Event::MouseButtonPressed {button, x, y} = event {
				if button == Button::Left {
					if board.place(((x/64) as u8, (y/64) as u8), next_piece) {
						score.update_score(&board);
						next_piece = next_piece.opposite();
					}
				}
				else if button == Button::Right {
					match next_piece {
						Piece::BLACK => println!("Black has passed"),
						Piece::WHITE => println!("White has passed"),
					}
					next_piece = next_piece.opposite();
				}
			}
		}

		window.clear(&Color::rgb(100, 200, 100));
		window.draw(&board);
		window.draw(&score);
		window.display();
	}
}
