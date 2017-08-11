use sfml::window::{ContextSettings, VideoMode, Event, style};
use sfml::window::mouse::Button;
use sfml::graphics::{Color, RenderTarget, RenderWindow, Rect};
use std::sync::Arc;
use cli::{DrawableBoard, DrawableScore, SCORE_HEIGHT, NetHandler};
use board::*;
use score::*;
use packets::*;

pub struct Game {
	piece: Piece,
	opponent: ClientId,
	nethandler: Arc<NetHandler>,
	board: DrawableBoard,
	score: DrawableScore,
	window: RenderWindow
}

impl Game {
	pub fn new(nethandler: Arc<NetHandler>, piece: Piece, opponent: ClientId) -> Game {
		// Create the board this game will be played in.
		let board = DrawableBoard::new(Board::new()).unwrap();

		// Create the window for the game.
		let mut window = RenderWindow::new(VideoMode::new(board.size(), board.size() + SCORE_HEIGHT, 32), "SFML Othello", style::CLOSE, &ContextSettings::default()).unwrap();
		window.set_framerate_limit(30);

		// Create the Score Bar
		let score_size = Rect::<u32> {
			left: 0,
			top: board.size(),
			width: board.size(),
			height: SCORE_HEIGHT
		};

		let score = DrawableScore::new(Score::new(&board), score_size);

		Game {
			piece: piece,
			opponent: opponent,
			nethandler: nethandler,
			board: board,
			score: score,
			window: window
		}
	}

	pub fn update(&mut self) {
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
	pub fn handle_packet(&mut self, packet: &Packet) -> bool {
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

	pub fn draw(&mut self) {
		self.window.clear(&Color::rgb(100, 200, 100));
		self.window.draw(&self.board);
		self.window.draw(&self.score);
		self.window.display();
	}
}
