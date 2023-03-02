use rand::Rng;
use sdl2::rect::Point;
use vector2d::Vector2D;

use crate::{
    enums::{game_object::GameObject, unit_type::UnitType},
    structs::{
        ent::{Ent, EntParentType, Owner},
        ore_patch::{OrePatch, OreType},
        unit::Unit,
        world::World,
        world_info::WorldInfo,
    },
};

use super::values::{MAP_HEIGHT, MAP_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH};

// Debug method; spawns some ents for testing
pub fn spawn_debug_ents(n: i32, world: &mut World, world_info: &mut WorldInfo) {
    let mut rng = rand::thread_rng();
    for i in 0..n {
        let new_ent = Ent::new(
            EntParentType::Unit,
            if i < n / 2 { Owner::Player } else { Owner::Cpu },
            100,
            Vector2D::<f32>::new(
                rng.gen_range(MAP_WIDTH / 2 + 25..MAP_WIDTH / 2 + SCREEN_WIDTH) as f32,
                rng.gen_range(MAP_HEIGHT / 2 + 25..MAP_HEIGHT / 2 + SCREEN_HEIGHT) as f32,
            ),
            Point::new(rng.gen_range(5..50), rng.gen_range(5..50)),
        );
        world_info.add_ent(&new_ent);
        world
            .game_objects
            .push(GameObject::Unit(new_ent, UnitType::Scout(Unit::new())));
    }

    let new_ent = Ent::new(
        EntParentType::OrePatch,
        Owner::Nature,
        100,
        Vector2D::<f32>::new(
            rng.gen_range(500..750) as f32,
            rng.gen_range(650..850) as f32,
        ),
        Point::new(rng.gen_range(5..50), rng.gen_range(5..50)),
    );
    world_info.add_ent(&new_ent);
    world.game_objects.push(GameObject::OrePatch(
        new_ent,
        OrePatch::new(OreType::Blue, 10, 5),
    ));
}
