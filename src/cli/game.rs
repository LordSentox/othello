use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::window::{ContextSettings, VideoMode, Event, style};
use sfml::window::mouse::Button;
use board::*;
use score::Score;

/// Structure representing a game with another player.
pub struct Game {
	board: Board,
	score: Score,
	window: RenderWindow
}

impl Game {
	/// Create a new GameHandler. This does not start a game immediately.
	pub fn new() -> Game {
		// Create a window for the board-view.
		// TODO: On OSX, make sure this is always done in the main-thread, because windw creation
		// will otherwise always fail.
		let mut window = RenderWindow::new(VideoMode::new(512, 532, 32), "SFML Othello", style::CLOSE, &ContextSettings::default()).unwrap();
		window.set_framerate_limit(30);

		let board = Board::new();
		let score = Score::new(&board);

		Game {
			board: board,
			score: score,
			window: window
		}
	}

	pub fn draw(&mut self) {
		unimplemented!();
	}
}
