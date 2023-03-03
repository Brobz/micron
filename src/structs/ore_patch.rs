use rand::Rng;
use sdl2::{rect::Point, render::Canvas, video::Window};
use vector2d::Vector2D;

use crate::{
    consts::{
        helper::draw_rect_selection_border,
        values::{BLACK_RGB, BLUE_RGB, YELLOW_RGBA_WEAK},
    },
    enums::game_object::GameObject,
};

use super::{
    ent::{Ent, EntParentType},
    ore::Ore,
    world_info::WorldInfo,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum OreType {
    Blue,
}
pub struct OrePatch {
    ore_type: OreType,
    density: u32,
    richness: f32,
}

impl OrePatch {
    pub fn new(ore_type: OreType, density: u32, richness: f32) -> Self {
        Self {
            ore_type,
            density,
            richness,
        }
    }

    pub fn tick(&mut self, ent: &mut Ent, world_info: &mut WorldInfo) -> Option<GameObject> {
        // Update local HP based on world_info data
        // If not found there, then unit is dead
        ent.hp = world_info.get_ent_hp(ent).unwrap_or(0.0);

        // If dead, return early
        if ent.hp <= 0.0 {
            return None;
        }

        // Calculate ratio of damage taken and drop ore accordingly
        let health_left = ent.hp / ent.max_hp as f32;
        if health_left != 1.0 && (health_left * 100.0) as u32 % self.density == 0 {
            // TODO: Limit how many times this triggers per chunk  (chunks are correct)
            dbg! {(health_left, (health_left * 100.0) as u32 % self.density)};
            return Some(self.drop_new_ore(ent, world_info));
        }

        None
    }

    fn drop_new_ore(&self, ent: &Ent, world_info: &mut WorldInfo) -> GameObject {
        let mut rng = rand::thread_rng();
        let new_ent = Ent::new(
            EntParentType::Ore,
            ent.owner,
            (self.richness * 100.0) as u32,
            ent.position
                - Vector2D::<f32>::new(
                    rng.gen_range(
                        -(ent.get_rect().width() as i32) * 2..ent.get_rect().height() as i32 * 2,
                    ) as f32,
                    rng.gen_range(
                        -(ent.get_rect().width() as i32) * 2..ent.get_rect().height() as i32 * 2,
                    ) as f32,
                ),
            Point::new(
                (self.richness * 100.0) as i32,
                (self.richness * 100.0) as i32,
            ),
            BLUE_RGB,
        );
        world_info.add_ent(&new_ent);
        GameObject::Ore(new_ent, Ore::new(self.ore_type, self.richness))
    }
    pub fn draw(&self, ent: &mut Ent, canvas: &mut Canvas<Window>) {
        // If dead, return early
        if ent.hp <= 0.0 {
            return {};
        }
        // If selected, draw selection border
        if ent.selected() {
            let border_color = YELLOW_RGBA_WEAK;
            draw_rect_selection_border(canvas, &ent.get_rect(), border_color);
        }

        // Draw self (if alive)
        canvas.set_draw_color(ent.color);
        let rect = ent.get_rect();
        canvas.fill_rect(rect).ok();
        canvas.set_draw_color(BLACK_RGB);
        canvas.draw_rect(rect).ok();
    }
}
