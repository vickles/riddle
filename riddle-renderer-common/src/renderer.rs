use crate::{vertex::Vertex, CommonSprite};
use riddle_common::Color;
use riddle_image::ImageError;
use riddle_math::{Rect, Vector2};
use riddle_platform_common::WindowId;

/// The root object of a renderer implementation, associated with a single display.
pub trait CommonRenderer: Sized {
	type RenderContext: RenderContext<Self>;
	type Sprite: CommonSprite<Self>;
	type Texture;
	type Shader;
	type SpriteFont;
	type Error: std::error::Error + std::convert::From<ImageError>;

	fn dimensions(&self) -> Vector2<f32>;
	fn window_id(&self) -> WindowId;

	fn render<R, F>(&self, f: F) -> Result<R, Self::Error>
	where
		F: FnOnce(&mut Self::RenderContext) -> Result<R, Self::Error>;
}

/// The context provided to render callbacks
pub trait RenderContext<R: CommonRenderer> {
	/// Replace the current world transform.
	fn set_transform(&mut self, transform: mint::ColumnMatrix4<f32>) -> Result<(), R::Error>;

	/// Fill the target with a flat color.
	fn clear(&mut self, color: Color<f32>) -> Result<(), R::Error>;

	/// Draw a `Renderable` to the target with the current world transform.
	fn draw(&mut self, renderable: &Renderable<'_, R>) -> Result<(), R::Error>;

	/// Draw a solid rect with the given color.
	fn fill_rect(&mut self, rect: &Rect<f32>, color: Color<f32>) -> Result<(), R::Error>;

	/// Consume the context and present any outstanding draw calls.
	fn present(self) -> Result<(), R::Error>;
}

pub struct Renderable<'a, R: CommonRenderer> {
	pub texture: R::Texture,
	pub shader: R::Shader,
	pub verts: &'a [Vertex],
	pub indices: &'a [u16],
}
