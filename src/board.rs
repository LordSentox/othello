// A piece that might be placed on the board.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Piece {
	Black,
	White
}

impl Piece {
	// Return the complementary piece of the one this is called on.
	pub fn opposite(&self) -> Piece {
		match self {
			&Piece::Black => Piece::White,
			&Piece::White => Piece::Black
		}
	}
}

pub struct Board {
	// Contains the info about the 8x8 board.
	squares: Vec<Vec<Option<Piece>>>,
	turn: Piece
}

impl Board {
	/// Create a new board that
	pub fn new() -> Board {
		// Program in the starting information.
		let mut squares: Vec<Vec<Option<Piece>>> = vec![vec![None; 8]; 8];
		squares[3][3] = Some(Piece::White);
		squares[4][4] = Some(Piece::White);
		squares[3][4] = Some(Piece::Black);
		squares[4][3] = Some(Piece::Black);

		Board {
			squares: squares,
			turn: Piece::Black
		}
	}

	/// Gets all opportunities for the provided piece. It is irrellevant whos turn it is, the board
	/// will check everything as if it were the turn of the provided piece.
	pub fn opportunities(&self, piece: Piece) -> Vec<(u8, u8)> {
		// First, create a vector of all places where there is no piece. This will speed up the
		// process a little bit, since can_place() is a fairly costly operation.
		let mut opportunities: Vec<(u8, u8)> = Vec::new();
		for x in 0..8 {
			for y in 0..8 {
				if self.squares[x][y].is_none() {
					opportunities.push((x as u8, y as u8));
				}
			}
		}

		opportunities.retain(|&(ref x, ref y)| { self.can_place((*x, *y), piece) });
		opportunities
	}

	/// Check if a stone with the given colour can be placed at the point in
	/// question.
	pub fn can_place(&self, (x, y): (u8, u8), piece: Piece) -> bool {
		!self.affected_directions((x, y), piece).is_empty()
	}

	/// Returns a vector of directions that would be affected, should the piece
	/// be placed at the square in question.
	pub fn affected_directions(&self, (x, y): (u8, u8), piece: Piece) -> Vec<(i8, i8)> {
		let dirs: Vec<(i8, i8)> = vec![(-1, 0), (-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1)];
		let mut aff_dirs: Vec<(i8, i8)> = Vec::new();

		for (dx, dy) in dirs {
			let mut first = true;
			let mut done = false;
			let mut err_occured = false;

			// Check the first stone next to the one in question.
			let (mut cur_x, mut cur_y) = (x as i8, y as i8);
			while !err_occured && !done {
				cur_x += dx;
				cur_y += dy;
				if cur_x < 0 || cur_y < 0 || cur_x >= 8 || cur_y >= 8 {
					err_occured = true;
				}
				else if Some(piece) == self.squares[cur_x as usize][cur_y as usize] {
					// The current piece is the same as the one to check.
					if first { err_occured = true; }
					else { done = true; }
				}
				else if None == self.squares[cur_x as usize][cur_y as usize] {
					// There is no further piece in the direction. The line has
					// not been closed off properly.
					err_occured = true;
				}

				first = false;
			}

			if first { err_occured = true }
			if !err_occured && done {
				aff_dirs.push((dx, dy));
			}
		}

		aff_dirs
	}

	pub fn place(&mut self, (x, y): (u8, u8), piece: Piece) -> bool {
		if self.turn != piece {
			return false;
		}

		// Check if the given position is a valid position on the board.
		if x >= 8 || y >= 8 {
			return false;
		}

		// Cannot place a piece in case there is already one on the square.
		if let Some(_) = self.squares[x as usize][y as usize] {
			return false;
		}

		let dirs = self.affected_directions((x, y), piece);

		if dirs.is_empty() {
			return false;
		}

		// Go through the directions and change the colour of every single piece
		// that is the different colour until you reach the first of the same.
		// If an error occurs here it is likely in affected_directions()
		for (dx, dy) in dirs {
			let (mut cur_x, mut cur_y) = (x as i8 + dx, y as i8 + dy);
			assert!(cur_x >= 0 && cur_y >= 0 && cur_x < 8 && cur_y < 8);

			// Flip all the pieces in the current direction.
			while Some(piece.opposite()) == self.squares[cur_x as usize][cur_y as usize] {
				self.squares[cur_x as usize][cur_y as usize] = Some(piece);

				cur_x += dx;
				cur_y += dy;
				// If this assertion fails, then the fault is in affected directions.
				assert!(cur_x >= 0 && cur_y >= 0 && cur_x < 8 && cur_y < 8);
			}
		}

		// Place the actual piece on the board.
		self.squares[x as usize][y as usize] = Some(piece);

		// Since the piece could be set successfully, it is now the other players turn.
		self.turn = self.turn.opposite();

		true
	}

	pub fn turn(&self) -> Piece {
		self.turn
	}

	pub fn pass(&mut self) {
		self.turn = self.turn.opposite();
	}

	pub fn squares(&self) -> &Vec<Vec<Option<Piece>>> {
		&self.squares
	}

	pub fn squares_mut(&mut self) -> &mut Vec<Vec<Option<Piece>>> {
		&mut self.squares
	}

	pub fn print(&self) {
		for y in 0..8 {
			for x in 0..8 {
				match self.squares[x][y] {
					Some(Piece::White) => print!("W"),
					Some(Piece::Black) => print!("B"),
					None => print!("-")
				};
			}
			println!("");
		}
	}
}
