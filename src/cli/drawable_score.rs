use std::ops::{Deref, DerefMut};
use sfml::graphics::{CircleShape, Color, Drawable, Rect, RectangleShape, RenderTarget, RenderStates, Shape, Transformable};
use sfml::system::Vector2f;

use board::{Piece, Board};
use score::Score;

pub struct DrawableScore {
	bounds: Rect<u32>,
	inner: Score
}

impl DrawableScore {
	pub fn new(score: Score, bounds: Rect<u32>) -> DrawableScore {
		DrawableScore {
			bounds: bounds,
			inner: score
		}
	}
}

impl Drawable for DrawableScore {
	fn draw<'se, 'tex, 'sh, 'shte>(&'se self, target: &mut RenderTarget, _: RenderStates<'tex, 'sh, 'shte>)
	where 'se: 'sh {
		let (white, black) = self.get_score();

		let white_length = self.bounds.width as f32 * white as f32 / (white + black) as f32;
		let mut white_bar = RectangleShape::with_size(&Vector2f::new(white_length, self.bounds.height as f32));
		let mut black_bar = RectangleShape::with_size(&Vector2f::new(self.bounds.width as f32, self.bounds.height as f32));

		white_bar.set_position2f(self.bounds.left as f32, self.bounds.top as f32);
		black_bar.set_position(&white_bar.position());

		white_bar.set_fill_color(&Color::white());
		black_bar.set_fill_color(&Color::black());

		// The white bar is the only one that gets resized. It will simply be
		// rendered over the black bar.
		target.draw(&black_bar);
		target.draw(&white_bar);
	}
}

impl Deref for DrawableScore {
	type Target = Score;

	fn deref(&self) -> &Score {
		&self.inner
	}
}

impl DerefMut for DrawableScore {
	fn deref_mut(&mut self) -> &mut Score {
		&mut self.inner
	}
}
