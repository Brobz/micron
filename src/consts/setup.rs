use rand::{rngs::ThreadRng, Rng};
use sdl2::rect::Point;
use vector2d::Vector2D;

use crate::structs::{ent::Ent, unit::Unit, world::World, world_info::WorldInfo};

use super::values::{MAP_HEIGHT, MAP_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH};

// Debug method; spawns some ents for testing
pub fn spawn_debug_ents(rng: &mut ThreadRng, world: &mut World, world_info: &mut WorldInfo) {
    for _ in 1..10 {
        let new_ent = Ent::new(
            100,
            Vector2D::<f32>::new(
                rng.gen_range(MAP_WIDTH / 2 + 25..MAP_WIDTH / 2 + SCREEN_WIDTH) as f32,
                rng.gen_range(MAP_HEIGHT / 2 + 25..MAP_HEIGHT / 2 + SCREEN_HEIGHT) as f32,
            ),
            Point::new(rng.gen_range(5..50), rng.gen_range(5..50)),
        );
        world_info.add_ent(&new_ent);
        world.units.push(Unit::new(new_ent));
    }
}
