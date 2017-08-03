use std::ops::{Deref, DerefMut};
use board::*;
use config::CONFIG;
use sfml::graphics::{Color, Texture, Drawable, RenderTarget, RenderStates, Sprite, Transformable};

/// Client ontly wrapper for the board, where the board has to be rendered.
pub struct DrawableBoard {
	board_tex: Texture,
	white_piece_tex: Texture,
	black_piece_tex: Texture,
	shadow_tex: Option<Texture>, // The shadow that will be drawn around each piece.
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

		// Check if the board texture is quadratic.
		if board_tex.size().x != board_tex.size().y {
			println!("Could not load board texture. Texture must be quadratic.");
			return None;
		}

		let white_piece_tex = match Texture::from_file(&CONFIG.graphics.white_piece) {
			Some(t) => t,
			None => {
				println!("Could not load white piece texture.");
				return None;
			}
		};

		// Check if the white piece texture is quadratic.
		if white_piece_tex.size().x != white_piece_tex.size().y {
			println!("Could not load white piece texture. Texture must be quadratic.");
			return None;
		}

		let black_piece_tex = match Texture::from_file(&CONFIG.graphics.black_piece) {
			Some(t) => t,
			None => {
				println!("Could not load black piece texture.");
				return None;
			}
		};

		// Check if the black piece texture is quadratic.
		if black_piece_tex.size().x != black_piece_tex.size().y {
			println!("Could not load black piece texture. Texture must be quadratic.");
			return None;
		}

		let shadow_tex = {
			// Check if shadow is available.
		 	if let Some(ref file) = CONFIG.graphics.shadow {
				match Texture::from_file(file) {
					Some(t) => Some(t),
					None => {
						println!("Could not load shadow texture. Falling back to shadowless version.");
						None
					}
				}
			}
			else { None }
		};

		// Buffer drawable board.
		let db = DrawableBoard {
			board_tex: board_tex,
			white_piece_tex: white_piece_tex,
			black_piece_tex: black_piece_tex,
			shadow_tex: shadow_tex,
			inner: board
		};

		// Lastly, check if the white and black pieces are suitable for the board.
		if db.white_piece_tex.size().x as u16 != db.piece_size() {
			println!("White piece does not have the right dimensions ({}x{}).", db.piece_size(), db.piece_size());
			return None;
		}

		if db.black_piece_tex.size().x as u16 != db.piece_size() {
			println!("Black piece does not have the right dimensions ({}x{}).", db.piece_size(), db.piece_size());
			return None;
		}

		// Everything loaded, all checks have passed.
		Some(db)
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

		// Create a vector to hold all the stones and optional shadows, so that
		// all shadows can later be drawn before all sprites.
		let mut pieces: Vec<(Sprite, Option<Sprite>)> = Vec::new();

		for x in 0..8 {
			for y in 0..8 {
				// Check if a piece is at this position and create it.
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
				let pos_x = (x as u16 * size) as f32;
				let pos_y = (y as u16 * size) as f32;
				sprite.set_position2f(pos_x, pos_y);

				// If no shadows are available, just add the current piece to the
				// rendering Vector and continue.
				if self.shadow_tex.is_none() {
					pieces.push((sprite, None));
					continue;
				}

				// Shadow is available, so add it to the piece and then push.
				let shadow_tex = self.shadow_tex.as_ref().unwrap();

				// Calculate the position of the shadow, since the shadow is most
				// likely a different and greater size than the piece.
				let shadow_middle: (u32, u32) = (shadow_tex.size().x / 2, shadow_tex.size().y / 2);
				let piece_middle:  (u32, u32) = (self.white_piece_tex.size().x / 2, self.white_piece_tex.size().y / 2);

				let offset_x = piece_middle.0 as f32 - shadow_middle.0 as f32;
				let offset_y = piece_middle.1 as f32 - shadow_middle.1 as f32;

				let mut shadow_sprite = Sprite::with_texture(shadow_tex);
				shadow_sprite.set_position2f(pos_x + offset_x, pos_y + offset_y);

				pieces.push((sprite, Some(shadow_sprite)));
			}
		}

		// Draw the shadows first, if available.
		if self.shadow_tex.is_some() {
			for &(_, ref shadow) in &pieces {
				match shadow {
					&Some(ref shadow) => target.draw(shadow),
					&None => {}
				}
			}
		}

		// Draw the pieces to the screen.
		for (piece, _) in pieces {
			target.draw(&piece);
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
