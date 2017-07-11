#![feature(associated_consts)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate sfml;
extern crate toml;

pub mod board;
pub mod cli;
pub mod score;
pub mod packets;

use board::{Board, Piece};
use score::{Score};
use packets::*;

use sfml::window::{ContextSettings, VideoMode, Event, style};
use sfml::window::mouse::Button;
use sfml::graphics::{Color, RenderTarget, RenderWindow};

use std::net::TcpStream;
use std::io::Write;

use std::str::FromStr;

fn main() {
	// Connect to a server
	println!("Welcome to othello.");
	let args: Vec<String> = std::env::args().collect();
	if args.len() < 2 {
		panic!("Could not start client. Please provide the IP of the server you want to connect to.");
	}

	let server_ip = args[1].clone();

	println!("Connecting to server.. IP: {}", server_ip);
	let mut stream = TcpStream::connect(server_ip).expect("Failed to connect to server!");

	// Send a test packet.
	let p = Packet::RequestClientList;
	assert!(p.write_to_stream(&mut stream));
	// Listen to the response from the server.
	loop {
		match Packet::read_from_stream(&mut stream) {
			(Some(p), true) => { println!("Received response to test: {:?}", p); break; },
			(None, true) => println!("Error receiving packet."),
			(_, false) => panic!("Connection has been closed by the server.")
		}

	}

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
