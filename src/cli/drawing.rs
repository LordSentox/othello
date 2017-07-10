use sfml::graphics::{CircleShape, Color, Drawable, RectangleShape, RenderTarget, RenderStates, Shape, Transformable};
use sfml::system::Vector2f;

use board::{Piece, Board};
use score::Score;

impl Drawable for Board {
	fn draw<'se, 'tex, 'sh, 'shte>(&'se self, target: &mut RenderTarget, _: RenderStates<'tex, 'sh, 'shte>)
	where 'se: 'sh {
		for x in 0..8 {
			for y in 0..8 {
				// Draw the underlying board.
				let mut board_rect = RectangleShape::with_size(&Vector2f::new(64., 64.));
				board_rect.set_outline_thickness(-1.);
				board_rect.set_outline_color(&Color::rgb(55, 55, 55));
				board_rect.set_fill_color(&Color::transparent());
				board_rect.set_position(&Vector2f::new(64. * x as f32, 64. * y as f32));
				target.draw(&board_rect);

				// Draw the stone if one is on the board at the correct coordinates.
				let mut circle_shape = match self.squares()[x][y] {
					Some(Piece::WHITE) => {
						let mut temp = CircleShape::new();
						temp.set_fill_color(&Color::white());
						temp
					},
					Some(Piece::BLACK) => {
						let mut temp = CircleShape::new();
						temp.set_fill_color(&Color::black());
						temp
					},
					None => continue
				};

				circle_shape.set_radius(32.);
				circle_shape.set_position(&Vector2f::new(64. * x as f32, 64. * y as f32));

				target.draw(&circle_shape);
			}
		}
	}
}

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
