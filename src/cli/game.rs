use sfml::window::{ContextSettings, Event, VideoMode, style};
use sfml::window::mouse::Button;
use sfml::graphics::{Color, RenderTarget, RenderWindow, Rect};
use std::sync::Arc;
use cli::{DrawableBoard, DrawableScore, SCORE_HEIGHT, NetHandler};
use board::*;
use score::*;
use packets::*;

pub trait Game {
	fn handle_events(&mut self);
	fn handle_packet(&mut self, packet: &Packet) -> bool;
	fn running(&self) -> bool;
	fn draw(&mut self);
}

fn initialise_graphics() -> (DrawableBoard, DrawableScore, RenderWindow) {
	// Create the board this game will be played in.
	let board = DrawableBoard::new(Board::new()).unwrap();

	// Create the window for the game.
	let mut window = RenderWindow::new(VideoMode::new(board.size(), board.size() + SCORE_HEIGHT, 32), "SFML Othello", style::CLOSE, &ContextSettings::default()).unwrap();
	window.set_framerate_limit(20);

	// Create the Score Bar
	let score_size = Rect::<u32> {
		left: 0,
		top: board.size(),
		width: board.size(),
		height: SCORE_HEIGHT
	};

	let score = DrawableScore::new(Score::new(&board), score_size);

	(board, score, window)
}

pub struct OfflineGame {
	board: DrawableBoard,
	score: DrawableScore,
	window: RenderWindow,
	running: bool
}


impl OfflineGame {
	pub fn new() -> OfflineGame {
		let (board, score, window) = initialise_graphics();

		OfflineGame {
			board: board,
			score: score,
			window: window,
			running: true
		}
	}
}

impl Game for OfflineGame {
	fn handle_events(&mut self) {
		for event in self.window.events() {
			if let Event::Closed = event {
				self.running = false;
			}
			else if let Event::MouseButtonPressed {button, x, y} = event {
				if button == Button::Left {
					let pos = self.board.piece_index(x as u32, y as u32);
					let turn = self.board.turn();
					if self.board.place(pos, turn) {
						self.score.update_score(&self.board);
					}
				}
				else if button == Button::Right {
					println!("Passing has not been implemented yet");
				}
			}
		}
	}

	fn handle_packet(&mut self,  _: &Packet) -> bool { false }

	fn running(&self) -> bool {
		self.running
	}

	fn draw(&mut self) {
		self.window.clear(&Color::rgb(100, 200, 100));
		self.window.draw(&self.board);
		self.window.draw(&self.score);
		self.window.display();
	}
}

pub struct OnlineGame {
	piece: Piece,
	opponent: ClientId,
	nethandler: Arc<NetHandler>,
	board: DrawableBoard,
	score: DrawableScore,
	window: RenderWindow
}

impl OnlineGame {
	pub fn new(nethandler: Arc<NetHandler>, piece: Piece, opponent: ClientId) -> OnlineGame {
		let (board, score, window) = initialise_graphics();

		OnlineGame {
			piece: piece,
			opponent: opponent,
			nethandler: nethandler,
			board: board,
			score: score,
			window: window
		}
	}
}

impl Game for OnlineGame {
	fn handle_events(&mut self) {
		for event in self.window.events() {
			if let Event::Closed = event {
				return;
			}
			else if let Event::MouseButtonPressed {button, x, y} = event {
				if button == Button::Left {
					let pos = self.board.piece_index(x as u32, y as u32);
					if self.board.place(pos, self.piece) {
						self.score.update_score(&self.board);

						// Send the move to the server.
						self.nethandler.send(&Packet::PlacePiece(self.opponent, pos.0, pos.1));
					}
				}
				else if button == Button::Right {
					println!("Passing has not been implemented yet");
				}
			}
		}
	}

	/// Handle the packet. Returns true if the packet was part of this game and doesn't have to be
	/// passed on to another game anymore.
	fn handle_packet(&mut self, packet: &Packet) -> bool {
		match packet {
			&Packet::PlacePiece(opponent, x, y) => {
				if self.opponent != opponent {
					return false;
				}

				self.board.place((x, y), self.piece.opposite());
				self.score.update_score(&self.board);
				true
			}
			_ => false
		}
	}

	fn running(&self) -> bool {
		true
	}

	fn draw(&mut self) {
		self.window.clear(&Color::rgb(100, 200, 100));
		self.window.draw(&self.board);
		self.window.draw(&self.score);
		self.window.display();
	}
}
