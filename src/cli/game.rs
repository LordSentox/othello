use sfml::window::{ContextSettings, VideoMode, Event, style};
use sfml::window::mouse::Button;
use sfml::graphics::{Color, RenderTarget, RenderWindow, Rect};
use cli::{DrawableBoard, DrawableScore, SCORE_HEIGHT};
use board::*;
use score::*;
use packets::*;

pub struct Game {
	piece: Piece,
	opponent: ClientId,
	board: DrawableBoard,
	score: DrawableScore,
	window: RenderWindow
}

impl Game {
	pub fn new(piece: Piece, opponent: ClientId) -> Game {
		// Create the board this game will be played in.
		let mut board = DrawableBoard::new(Board::new()).unwrap();

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
		let mut score = DrawableScore::new(Score::new(&board), score_size);

		Game {
			piece: piece,
			opponent: opponent,
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
					}
				}
				else if button == Button::Right {
					println!("Passing has not been implemented yet");
				}
			}
		}
	}

	pub fn draw(&mut self) {
		self.window.clear(&Color::rgb(100, 200, 100));
		self.window.draw(&self.board);
		self.window.draw(&self.score);
		self.window.display();
	}
}
