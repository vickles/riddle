use crate::{math::*, *};

use std::rc::Rc;

/// A sprite represents an image texture, along with an axis aligned rect to select which
/// part of the texture the sprite represents.
///
/// Sprites store a reference to the Renderer that constructed it.
pub struct Sprite {
    renderer: Rc<Renderer>,
    texture: Rc<Texture>,
    source_rect: Rect<f32>,
}

impl Sprite {
    /// Construct a new sprite from an image. The image contents are copied to a texture
    /// in RGBA8 format. The entire image will be used
    pub fn new_from_image(
        renderer: &Rc<Renderer>,
        img: image::Image,
    ) -> Result<Sprite, RendererError> {
        let texture = Texture::from_image(&renderer.device, &renderer.queue, img)?;
        Ok(Self::from_texture(renderer, texture.into()))
    }

    pub(super) fn from_texture(renderer: &Rc<Renderer>, texture: Rc<Texture>) -> Sprite {
        let dimensions = texture.dimensions.convert();
        Self::from_texture_with_bounds(
            renderer,
            texture,
            Rect {
                location: Vector2 { x: 0.0, y: 0.0 },
                dimensions,
            },
        )
    }

    pub(super) fn from_texture_with_bounds(
        renderer: &Rc<Renderer>,
        texture: Rc<Texture>,
        source_rect: Rect<f32>,
    ) -> Sprite {
        Sprite {
            renderer: renderer.clone(),
            texture: texture,
            source_rect,
        }
    }

    pub fn subsprite(&self, source_rect: &Rect<f32>) -> Sprite {
        let location =
            glam::Vec2::from(self.source_rect.location) + glam::Vec2::from(source_rect.location);

        Sprite {
            renderer: self.renderer.clone(),
            texture: self.texture.clone(),
            source_rect: Rect {
                location: location.into(), // Vector2::from(Into::<mint::Point2<f32>>::into(location)),
                dimensions: source_rect.dimensions,
            },
        }
    }

    pub fn render(&self, args: &SpriteRenderCommand) -> Result<(), RendererError> {
        let rot: glam::Mat2 = glam::Mat2::from_angle((args.angle / 180.0) * std::f32::consts::PI);
        let Vector2 {
            x: tex_width,
            y: tex_height,
        } = self.texture.dimensions;

        let location: glam::Vec2 = args.location.into();
        let pivot: glam::Vec2 = args.pivot.into();

        let scale = glam::Mat2::from_scale(args.scale.into());

        let pos_topleft = glam::vec2(0.0, 0.0) - pivot;
        let pos_topright = pos_topleft + glam::vec2(self.source_rect.dimensions.x, 0.0);
        let pos_bottomleft = pos_topleft + glam::vec2(0.0, self.source_rect.dimensions.y);
        let pos_bottomright = pos_bottomleft + glam::vec2(self.source_rect.dimensions.x, 0.0);

        let uv_top = self.source_rect.location.y / (tex_height as f32);
        let uv_left = self.source_rect.location.x / (tex_width as f32);
        let uv_bottom = uv_top + (self.source_rect.dimensions.y / (tex_height as f32));
        let uv_right = uv_left + (self.source_rect.dimensions.x / (tex_width as f32));

        let vertex_data = [
            Vertex::ptc(
                location + (rot * (scale * pos_topleft)),
                [uv_left, uv_top],
                &args.diffuse_color,
            ),
            Vertex::ptc(
                location + (rot * (scale * pos_bottomleft)),
                [uv_left, uv_bottom],
                &args.diffuse_color,
            ),
            Vertex::ptc(
                location + (rot * (scale * pos_bottomright)),
                [uv_right, uv_bottom],
                &args.diffuse_color,
            ),
            Vertex::ptc(
                location + (rot * (scale * pos_topright)),
                [uv_right, uv_top],
                &args.diffuse_color,
            ),
        ];

        let index_data: &[u16] = &[1, 2, 0, 2, 0, 3];

        self.renderer.stream_buffer.borrow_mut().stream_render(
            &self.renderer,
            &StreamRenderArgs {
                texture: self.texture.clone(),
                shader: self.renderer.default_shader.clone(),
            },
            &vertex_data[..],
            index_data,
        )
    }

    pub fn render_at<P: Into<Vector2<f32>>>(&self, location: P) -> Result<(), RendererError> {
        self.render(&SpriteRenderCommand {
            location: location.into(),
            ..Default::default()
        })
    }

    pub fn dimensions(&self) -> Vector2<f32> {
        self.source_rect.dimensions
    }

    pub fn pivot(&self, x: f32, y: f32) -> Vector2<f32> {
        Vector2 {
            x: self.source_rect.dimensions.x * x,
            y: self.source_rect.dimensions.y * y,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpriteRenderCommand {
    pub location: Vector2<f32>,
    pub pivot: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub angle: f32,
    pub diffuse_color: [f32; 4],
}

impl SpriteRenderCommand {
    pub fn new<T: Into<Vector2<f32>>>(location: T) -> Self {
        let mut args = Self::default();
        args.at(location);
        args
    }

    #[inline]
    pub fn at<T: Into<Vector2<f32>>>(&mut self, location: T) -> &mut Self {
        self.location = location.into();
        self
    }

    pub fn with_pivot<T: Into<Vector2<f32>>>(&mut self, pivot: T) -> &mut Self {
        self.pivot = pivot.into();
        self
    }

    pub fn with_scale<T: Into<Vector2<f32>>>(&mut self, scale: T) -> &mut Self {
        self.scale = scale.into();
        self
    }

    pub fn with_angle(&mut self, angle: f32) -> &mut Self {
        self.angle = angle;
        self
    }

    pub fn with_color(&mut self, color: [f32; 4]) -> &mut Self {
        self.diffuse_color = color;
        self
    }

    pub fn render(&self, sprite: &Sprite) -> Result<(), RendererError> {
        sprite.render(self)
    }
}

impl Default for SpriteRenderCommand {
    fn default() -> Self {
        SpriteRenderCommand {
            location: [0.0, 0.0].into(),
            pivot: [0.0, 0.0].into(),
            angle: 0.0,
            scale: [1.0, 1.0].into(),
            diffuse_color: [1.0; 4],
        }
    }
}