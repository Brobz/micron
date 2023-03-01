use sdl2::{
    rect::Rect,
    render::{Canvas, TextureCreator},
    ttf::Font,
    video::{Window, WindowContext},
};

use super::{camera::Camera, ui_element::UIElement};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TextLabel {}

impl TextLabel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
        camera: &Camera,
        ui_element: &UIElement,
    ) {
        // render a surface, and convert it to a texture bound to the canvas
        let surface = font.render(&ui_element.label).blended(ui_element.color);

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
                                    (ui_element.rect.x as f32 / camera.scale.x) as i32
                                        - camera.position.x,
                                    (ui_element.rect.y as f32 / camera.scale.y) as i32
                                        - camera.position.y,
                                    (ui_element.rect.width() as f32 / camera.scale.x) as u32,
                                    (ui_element.rect.height() as f32 / camera.scale.y) as u32,
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
}
