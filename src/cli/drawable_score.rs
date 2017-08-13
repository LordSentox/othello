use std::ops::{Deref, DerefMut};
use sfml::graphics::{CircleShape, Color, Drawable, Rect, RectangleShape, RenderTarget, RenderStates, Shape, Transformable};
use sfml::system::Vector2f;

use board::{Piece, Board};
use score::Score;
use config::CONFIG;

pub const SCORE_HEIGHT: u32 = 20;

pub struct DrawableScore<'a> {
	bounds: Rect<u32>,
	inner: Score<'a>
}

impl<'a> DrawableScore<'a> {
	pub fn new(score: Score<'a>, bounds: Rect<u32>) -> DrawableScore {
		DrawableScore {
			bounds: bounds,
			inner: score
		}
	}
}

impl<'a> Drawable for DrawableScore<'a> {
	fn draw<'se, 'tex, 'sh, 'shte>(&'se self, target: &mut RenderTarget, _: RenderStates<'tex, 'sh, 'shte>)
	where 'se: 'sh {
		let (white, black) = self.get_score();

		let white_length = self.bounds.width as f32 * white as f32 / (white + black) as f32;
		let mut white_bar = RectangleShape::with_size(&Vector2f::new(white_length, self.bounds.height as f32));
		let mut black_bar = RectangleShape::with_size(&Vector2f::new(self.bounds.width as f32, self.bounds.height as f32));

		white_bar.set_position2f(self.bounds.left as f32, self.bounds.top as f32);
		black_bar.set_position(&white_bar.position());

		white_bar.set_fill_color(&Color::rgb(CONFIG.graphics.white_score_colour[0], CONFIG.graphics.white_score_colour[1], CONFIG.graphics.white_score_colour[2]));
		black_bar.set_fill_color(&Color::rgb(CONFIG.graphics.black_score_colour[0], CONFIG.graphics.black_score_colour[1], CONFIG.graphics.black_score_colour[2]));

		// The white bar is the only one that gets resized. It will simply be
		// rendered over the black bar.
		target.draw(&black_bar);
		target.draw(&white_bar);
	}
}

impl<'a> Deref for DrawableScore<'a> {
	type Target = Score<'a>;

	fn deref(&self) -> &Score<'a> {
		&self.inner
	}
}

impl<'a> DerefMut for DrawableScore<'a> {
	fn deref_mut(&mut self) -> &mut Score<'a> {
		&mut self.inner
	}
}
