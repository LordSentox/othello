use board::{Board, Piece};

/// Bar to keep track of the current score between the two teams.
pub struct Score {
	white: u8,
	black: u8
}

impl Score {
	pub fn new(board: &Board) -> Score {
		let mut bar = Score {
			white: 0,
			black: 0
		};

		bar.update_score(&board);
		bar
	}

	// The score only gets updated, when this function is called, so that should
	// always be called after a stone has been placed on the board.
	pub fn update_score(&mut self, board: &Board) {
		self.white = 0;
		self.black = 0;

		for square in board.squares().iter().flat_map(|v| {v.iter()}) {
			match square {
				&Some(Piece::Black) => self.black += 1,
				&Some(Piece::White) => self.white += 1,
				_ => {}
			}
		}

		println!("Current score(w:b): {}:{}", self.white, self.black);
	}

	pub fn get_score(&self) -> (u8, u8) {
		(self.white, self.black)
	}
}
