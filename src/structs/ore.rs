use sdl2::{gfx::primitives::DrawRenderer, rect::Point, render::Canvas, video::Window};
use vector2d::Vector2D;

use crate::consts::{
    helper::draw_circle_selection_border,
    values::{BLACK_RGB, WHITE_RGB},
};

use super::{ent::Ent, ore_patch::OreType, world_info::WorldInfo};

pub struct Ore {
    ore_type: OreType,
    value: f32,
}

impl Ore {
    pub fn new(ore_type: OreType, value: f32) -> Self {
        Self { ore_type, value }
    }

    pub fn tick(&mut self, ent: &mut Ent, world_info: &mut WorldInfo) {
        // Update local HP based on world_info data
        // If not found there, then unit is dead
        ent.hp = world_info.get_ent_hp(ent).unwrap_or(0.0);

        // If dead, return early
        if ent.hp <= 0.0 {
            return;
        }

        // Update ent rect to have same dimensions as current radius
        let current_radius = self.get_radius(ent);
        ent.rect_size = Point::new(current_radius as i32, current_radius as i32);
    }

    pub fn draw(&self, ent: &mut Ent, canvas: &mut Canvas<Window>) {
        // If dead, return early
        if ent.hp <= 0.0 {
            return {};
        }
        // If selected, draw selection border
        if ent.selected() {
            let ent_rect_center = ent.get_rect().center();
            draw_circle_selection_border(
                canvas,
                Vector2D::<f32>::new(ent_rect_center.x as f32, ent_rect_center.y as f32),
                self.get_radius(ent),
                WHITE_RGB,
            );
        }

        let ent_rect_center = ent.get_rect().center();

        // Draw self (if alive)
        canvas.set_draw_color(ent.color);
        canvas
            .filled_circle(
                ent_rect_center.x as i16,
                ent_rect_center.y as i16,
                self.get_radius(ent),
                ent.color,
            )
            .ok();
        canvas.set_draw_color(BLACK_RGB);
        canvas
            .circle(
                ent_rect_center.x as i16,
                ent_rect_center.y as i16,
                self.get_radius(ent),
                ent.color,
            )
            .ok();
    }

    pub fn get_radius(&self, ent: &Ent) -> i16 {
        ((self.value * 100.0) * (ent.hp / ent.max_hp as f32)) as i16
    }
}
