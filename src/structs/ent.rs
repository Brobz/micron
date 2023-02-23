use bevy::prelude::*;

#[derive(Component)]
pub struct Ent {
    pub position: Vec3,
    pub max_hp: f32,
    pub hp: f32,
}

impl Ent {
    pub fn new(max_hp: f32) -> Self {
        Self {
            position: Vec3::new(-1.0, -1.0, 1.0),
            max_hp,
            hp: max_hp,
        }
    }
}
