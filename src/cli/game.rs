use sfml::window::{ContextSettings, VideoMode, Event, style};
use sfml::window::mouse::Button;
use sfml::graphics::{Color, RenderTarget, RenderWindow};
pub use board::*;
pub use score::*;

pub struct Game {
	board: Board,
	score: Score,
	window: RenderWindow
}

impl Game {
	pub fn new() -> Game {
		let mut window = RenderWindow::new(VideoMode::new(512, 532, 32), "SFML Othello", style::CLOSE, &ContextSettings::default()).unwrap();
		window.set_framerate_limit(30);

		// Create a test board and print its contents.
		let board = Board::new();

		// Create the Score Bar
		let score = Score::new(&board);

		Game {
			board: board,
			score: score,
			window: window
		}
	}
}
