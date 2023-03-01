use std::collections::HashMap;

use sdl2::{
    render::{Canvas, TextureCreator},
    ttf::Font,
    video::{Window, WindowContext},
};

use super::{
    camera::Camera, ui_element::UIElementID, ui_object::UIObject, world::World,
    world_info::WorldInfo,
};

pub struct UI {
    texture_creator: TextureCreator<WindowContext>,
    objects: HashMap<UIElementID, UIObject>,
}

impl UI {
    pub fn new(canvas: &mut Canvas<Window>) -> Self {
        Self {
            texture_creator: canvas.texture_creator(),
            objects: HashMap::new(),
        }
    }

    // Some UIObjects need to be updated every frame
    // If that is the case, it will happen here
    pub fn tick(&mut self, world: &World, _world_info: &WorldInfo) {
        for (ui_element_id, ui_object) in self.objects.iter_mut() {
            match ui_object {
                UIObject::TextLabel(ui_element, _) | UIObject::Button(ui_element, _) => {
                    match ui_element_id {
                        UIElementID::DEBUG_EntCount => {
                            ui_element.set_label(
                                "Ents: ".to_owned() + world.game_objects.len().to_string().as_str(),
                            );
                        }
                    }
                }
            }
        }
    }

    // Draw all currently visible UIObjects
    pub fn draw(&self, canvas: &mut Canvas<Window>, font: &Font, camera: &Camera) {
        // Draw Text Labels
        for ui_element in self.objects.values() {
            match ui_element {
                UIObject::TextLabel(ui_element, text_label) => {
                    text_label.draw(canvas, &self.texture_creator, font, camera, ui_element);
                }
                UIObject::Button(_, _) => todo!(),
            }
        }
    }

    pub fn add_ui_object(&mut self, new_ui_object: &UIObject) {
        match new_ui_object {
            UIObject::TextLabel(new_ui_element, _) | UIObject::Button(new_ui_element, _) => {
                self.objects
                    .entry(new_ui_element.id)
                    .or_insert_with(|| new_ui_object.clone());
            }
        }
    }

    pub fn _set_label_by_id(&mut self, id: UIElementID, new_label: String) {
        if let Some(ui_object) = self.objects.get_mut(&id) {
            match ui_object {
                UIObject::TextLabel(new_ui_element, _) | UIObject::Button(new_ui_element, _) => {
                    new_ui_element.set_label(new_label);
                }
            }
        }
    }
}
