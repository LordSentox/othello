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

	let server_ip = args[1].clone();

	println!("Connecting to server.. IP: {}", server_ip);
	let mut stream = TcpStream::connect(server_ip).expect("Failed to connect to server!");

	// Set the client name.
	println!("Setting name to [{}]", &args[2]);
	let p = Packet::ChangeNameRequest(args[2].clone());
	p.write_to_stream(&mut stream);

	loop {
		match Packet::read_from_stream(&mut stream) {
			Ok(Packet::ChangeNameResponse(true)) => { println!("Name has been set on the server."); break; },
			Ok(Packet::ChangeNameResponse(false)) => panic!("Name request has been denied by the server."),
			Ok(p) => println!("Received unexpected packet from server: {:?} .. ignoring", p),
			Err(err) => panic!("Error occured while sending name to server: {:?}", err)
		}
	}

	// Send a test packet.
	let p = Packet::RequestClientList;
	assert!(p.write_to_stream(&mut stream));
	println!("Requested client list.");
	// Listen to the response from the server.
	match Packet::read_from_stream(&mut stream) {
		Ok(p) => println!("Received response to test: {:?}", p),
		Err(PacketReadError::Closed) => panic!("Connection has been closed by the server."),
		Err(err) => println!("Error receiving packet. {:?}", err)
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
