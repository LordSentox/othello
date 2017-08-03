use std::ops::{Deref, DerefMut};
use board::*;
use config::CONFIG;
use sfml::graphics::{Color, Texture, Drawable, RenderTarget, RenderStates, Sprite, Transformable};

/// Client ontly wrapper for the board, where the board has to be rendered.
pub struct DrawableBoard {
	board_tex: Texture,
	white_piece_tex: Texture,
	black_piece_tex: Texture,
	inner: Board
}

impl DrawableBoard {
	pub fn new(board: Board) -> Option<DrawableBoard> {
		let board_tex = match Texture::from_file(&CONFIG.graphics.board) {
			Some(t) => t,
			None => {
				println!("Could not load board texture.");
				return None;
			}
		};

		// Check that the board texture is square_shaped.
		if board_tex.size().x != board_tex.size().y {
			println!("Could not load board texture. Board texture must be square shaped.");
			return None;
		}

		let white_piece_tex = match Texture::from_file(&CONFIG.graphics.white_piece) {
			Some(t) => t,
			None => {
				println!("Could not load white piece texture.");
				return None;
			}
		};

		let black_piece_tex = match Texture::from_file(&CONFIG.graphics.black_piece) {
			Some(t) => t,
			None => {
				println!("Could not load black piece texture.");
				return None;
			}
		};

		Some(DrawableBoard {
			board_tex: board_tex,
			white_piece_tex: white_piece_tex,
			black_piece_tex: black_piece_tex,
			inner: board
		})
	}

	/// The size (width and height are the same) of the entire board.
	pub fn size(&self) -> u32 {
		self.board_tex.size().x as u32
	}

	/// Get the size (width and height are the same) of an individual board piece.
	pub fn piece_size(&self) -> u16 {
		match CONFIG.graphics.square_size {
			Some(size) => size,
			None => (self.board_tex.size().x / 8) as u16
		}
	}

	/// Translates a position (usually of the mouse cursor) and translates it
	/// to the indices of the corresponding piece.
	pub fn piece_index(&self, x: u32, y: u32) -> (u8, u8) {
		let x = (x/64) as u8;
		let y = (y/64) as u8;

		(x, y)
	}
}

impl Drawable for DrawableBoard {
	fn draw<'se, 'tex, 'sh, 'shte>(&'se self, target: &mut RenderTarget, _: RenderStates<'tex, 'sh, 'shte>)
	where 'se: 'sh {
		// Draw the underlying board.
		let board_spr = Sprite::with_texture(&self.board_tex);
		target.draw(&board_spr);

		// TODO: Redo the loop with OpenGL-calls, which have way less overhead.
		for x in 0..8 {
			for y in 0..8 {
				// Draw the stone if one is on the board at the correct coordinates.
				let mut sprite = match self.squares()[x][y] {
					Some(Piece::WHITE) => {
						Sprite::with_texture(&self.white_piece_tex)
					},
					Some(Piece::BLACK) => {
						Sprite::with_texture(&self.black_piece_tex)
					},
					None => continue
				};

				let size = self.piece_size();
				sprite.set_position2f((x as u16*size) as f32, (y as u16*size) as f32);
				target.draw(&sprite);
			}
		}
	}
}

impl Deref for DrawableBoard {
	type Target = Board;

	fn deref(&self) -> &Board {
		&self.inner
	}
}

impl DerefMut for DrawableBoard {
	fn deref_mut(&mut self) -> &mut Board {
		&mut self.inner
	}
}
