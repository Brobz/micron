use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureCreator},
    ttf::Font,
    video::{Window, WindowContext},
};

use super::camera::Camera;

pub struct TextLabel {
    label: String,
    color: Color,
    rect: Rect,
}

impl TextLabel {
    pub fn new(label: String, color: Color, rect: Rect) -> Self {
        Self { label, rect, color }
    }

    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
        camera: &Camera,
    ) {
        // render a surface, and convert it to a texture bound to the canvas
        let surface = font.render(&self.label).blended(self.color);

        match surface {
            Ok(surface) => {
                let texture = texture_creator.create_texture_from_surface(&surface);

                match texture {
                    Ok(texture) => {
                        canvas
                            .copy(
                                &texture,
                                None,
                                Some(Rect::new(
                                    (self.rect.x as f32 / camera.scale.x) as i32
                                        - camera.position.x,
                                    (self.rect.y as f32 / camera.scale.y) as i32
                                        - camera.position.y,
                                    (self.rect.width() as f32 / camera.scale.x) as u32,
                                    (self.rect.height() as f32 / camera.scale.y) as u32,
                                )),
                            )
                            .ok();
                    }
                    Err(_) => todo!(),
                }
            }
            Err(_) => todo!(),
        }
    }

    pub fn set_label(&mut self, new_label: String) {
        self.label = new_label;
    }
}
