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

use std::io;

use board::{Board, Piece};
use score::{Score};

use sfml::window::{ContextSettings, VideoMode, Event, style};
use sfml::window::mouse::Button;
use sfml::graphics::{Color, Rect, RenderTarget, RenderWindow};

use cli::*;
use packets::Packet;

const SCORE_HEIGHT: u32 = 20;

fn main() {
	println!("Welcome to othello.");

	let login_name = match &CONFIG.network.login_name {
		&Some(ref name) => name.clone(),
		&None => {
			print!("Login: ");
			let mut login_name = String::new();
			io::stdin().read_line(&mut login_name).expect("Could not read login name. Aborting. {}");

			login_name
		}
	};

	// Create the connection to the server.
	// TODO: Handle the errors a little bit more graceful.
	let nethandler = NetHandler::connect((CONFIG.network.server_ip.as_str(), CONFIG.network.server_port), &login_name).expect("Could not connect to the server.");

	// Create a test board.
	let mut board = DrawableBoard::new(Board::new()).unwrap();

	// Create the window of the application
	let mut window = RenderWindow::new(VideoMode::new(board.size(), board.size() + SCORE_HEIGHT, 32), "SFML Othello", style::CLOSE, &ContextSettings::default()).unwrap();
	window.set_framerate_limit(30);


	// Create the Score Bar
	let score_size = Rect::<u32> {
		left: 0,
		top: board.size(),
		width: board.size(),
		height: SCORE_HEIGHT
	};
	let mut score = DrawableScore::new(Score::new(&board), score_size);

	// The player to set the first stone is black.
	let mut next_piece = Piece::Black;

	loop {
		for event in window.events() {
			if let Event::Closed = event {
				return;
			}
			else if let Event::MouseButtonPressed {button, x, y} = event {
				if button == Button::Left {
					let pos = board.piece_index(x as u32, y as u32);
					if board.place(pos, next_piece) {
						score.update_score(&board);
						next_piece = next_piece.opposite();
					}
				}
				else if button == Button::Right {
					match next_piece {
						Piece::Black => println!("Black has passed"),
						Piece::White => println!("White has passed"),
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
