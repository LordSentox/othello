use board::{Board, Piece};

/// Bar to keep track of the current score between the two teams.
pub struct Score<'a> {
	board: &'a Board,
	white: u8,
	black: u8
}

impl<'a> Score<'a> {
	pub fn score(board: &'a Board) -> Score<'a> {
		let mut white = 0;
		let mut black = 0;

		for square in board.squares().iter().flat_map(|v| {v.iter()}) {
			match square {
				&Some(Piece::Black) => black += 1,
				&Some(Piece::White) => white += 1,
				_ => {}
			}
		}

		Score {
			board: board,
			white: white,
			black: black
		}
	}

	pub fn white(&self) -> u8 {
		self.white
	}

	pub fn black(&self) -> u8 {
		self.black
	}

	/// Determine the winner of the game. Returns None in case the winner is unclear
	/// as of yet, otherwise the colour of the winner. This is strictly by stones,
	/// however. The ultimate winner may be decided somewhere else, for instance when
	/// one player forfeits the game.
	pub fn winner(&self) -> Option<Piece> {
		// The winner will be determined when both players are out of options, i.e. the board
		// cannot change any more.
		if !self.board.opportunities(Piece::Black).is_empty() {
			return None;
		}
		if !self.board.opportunities(Piece::White).is_empty() {
			return None;
		}

		// White has one when both parties have the same amount of stones, because black always has
		// the first move.
		if self.white >= self.black {
			Some(Piece::White)
		}
		else {
			Some(Piece::Black)
		}
	}

	pub fn get_score(&self) -> (u8, u8) {
		(self.white, self.black)
	}
}
