use sdl2::{gfx::primitives::DrawRenderer, render::Canvas, video::Window};

use crate::consts::{
    helper::draw_circle_selection_border,
    values::{BLACK_RGB, WHITE_RGB},
};

use super::{ent::Ent, ore_patch::OreType, world_info::WorldInfo};

pub struct Ore {
    ore_type: OreType,
    value: u32,
}

impl Ore {
    pub fn new(ore_type: OreType, value: u32) -> Self {
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
    }

    pub fn draw(&self, ent: &mut Ent, canvas: &mut Canvas<Window>) {
        // If dead, return early
        if ent.hp <= 0.0 {
            return {};
        }
        // If selected, draw selection border
        if ent.selected() {
            let border_color = WHITE_RGB;
            draw_circle_selection_border(canvas, ent.position, self.value as i16, border_color);
        }

        // Draw self (if alive)
        canvas.set_draw_color(ent.color);
        canvas
            .filled_circle(
                ent.position.x as i16,
                ent.position.y as i16,
                self.value as i16,
                ent.color,
            )
            .ok();
        canvas.set_draw_color(BLACK_RGB);
        canvas
            .circle(
                ent.position.x as i16,
                ent.position.y as i16,
                self.value as i16,
                ent.color,
            )
            .ok();
    }
}
