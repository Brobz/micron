use sdl2::{render::Canvas, video::Window};

use crate::consts::{
    helper::draw_selection_border,
    values::{BLACK_RGB, YELLOW_RGBA_WEAK},
};

use super::{ent::Ent, world_info::WorldInfo};

pub enum OreType {
    Blue,
}
pub struct OrePatch {
    ore_type: OreType,
    density: i32,
    richness: i32,
}

impl OrePatch {
    pub fn new(ore_type: OreType, density: i32, richness: i32) -> Self {
        Self {
            ore_type,
            density,
            richness,
        }
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
            let border_color = YELLOW_RGBA_WEAK;
            draw_selection_border(canvas, &ent.get_rect(), border_color);
        }

        // Draw self (if alive)
        canvas.set_draw_color(ent.color);
        let rect = ent.get_rect();
        canvas.fill_rect(rect).ok().unwrap_or_default();
        canvas.set_draw_color(BLACK_RGB);
        canvas.draw_rect(rect).ok();
    }
}
