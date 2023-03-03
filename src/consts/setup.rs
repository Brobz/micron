use rand::Rng;
use sdl2::rect::Point;
use vector2d::Vector2D;

use crate::{
    enums::game_object::GameObject,
    structs::{
        ent::{Ent, EntParentType, Owner},
        ore_patch::{OrePatch, OreType},
        unit::UnitParentType,
        world::World,
        world_info::WorldInfo,
    },
};

use super::{
    helper::new_unit,
    values::{BLUE_RGB, MAP_HEIGHT, MAP_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH},
};

// Debug method; spawns some ents for testing
pub fn spawn_debug_ents(n: i32, world: &mut World, world_info: &mut WorldInfo) {
    let mut rng = rand::thread_rng();
    let mut game_objects_to_add = Vec::<GameObject>::new();
    for i in 0..n {
        let unit_type = if i < n / 3 {
            UnitParentType::Scout
        } else if i < 2 * n / 3 {
            UnitParentType::Miner
        } else {
            UnitParentType::Collector
        };
        let position = Vector2D::<f32>::new(
            rng.gen_range(MAP_WIDTH / 2 + 25..MAP_WIDTH / 2 + SCREEN_WIDTH) as f32,
            rng.gen_range(MAP_HEIGHT / 2 + 25..MAP_HEIGHT / 2 + SCREEN_HEIGHT) as f32,
        );
        let owner = if i > n - 3 { Owner::Cpu } else { Owner::Player };
        game_objects_to_add.push(new_unit(world_info, unit_type, owner, position));
    }

    world.game_objects.append(&mut game_objects_to_add);

    let new_ent = Ent::new(
        EntParentType::OrePatch,
        Owner::Nature,
        100,
        Vector2D::<f32>::new(
            rng.gen_range(500..750) as f32,
            rng.gen_range(650..850) as f32,
        ),
        Point::new(rng.gen_range(50..100), rng.gen_range(50..100)),
        BLUE_RGB,
    );
    world_info.add_ent(&new_ent);
    world.game_objects.push(GameObject::OrePatch(
        new_ent,
        OrePatch::new(OreType::Blue, 10, 5),
    ));
}
