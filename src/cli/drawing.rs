use sfml::graphics::{CircleShape, Color, Drawable, RectangleShape, RenderTarget, RenderStates, Shape, Transformable};
use sfml::system::Vector2f;

use board::{Piece, Board};
use score::Score;

const LENGTH: f32 = 512.;
const HEIGHT: f32 = 20.;
impl Drawable for Score {
	fn draw<'se, 'tex, 'sh, 'shte>(&'se self, target: &mut RenderTarget, _: RenderStates<'tex, 'sh, 'shte>)
	where 'se: 'sh {
		let (white, black) = self.get_score();

		let mut white_bar = RectangleShape::with_size(&Vector2f::new(white as f32 / (white + black) as f32 * LENGTH, HEIGHT));
		let mut black_bar = RectangleShape::with_size(&Vector2f::new(LENGTH, HEIGHT));

		white_bar.set_position(&Vector2f::new(0., 512.));
		black_bar.set_position(&Vector2f::new(0., 512.));

		white_bar.set_fill_color(&Color::white());
		black_bar.set_fill_color(&Color::black());

		target.draw(&black_bar);
		target.draw(&white_bar);
	}
}
