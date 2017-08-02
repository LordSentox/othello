use board::Board;

/// Struct to handle games. It can start games and handle the game, when it
/// gets the necessary packets.
pub struct GameHandler {
	board: Option<Board>
}

impl GameHandler {
	/// Create a new GameHandler. This does not start a game immediately.
	pub fn new() -> GameHandler {
		GameHandler {
			board: None
		}
	}

	/// Check if there is already a game running.
	pub fn game_running(&self) -> bool {
		self.board.is_some()
	}

	pub fn start_game(&mut self, opponent: &str) -> bool {
		// Check if a game is already running
		if self.game_running() {
			return false;
		}

		// Start a new game between the two players.
		unimplemented!();
	}
}
