use bevy::prelude::*;

use crate::TIME_STEP;

#[derive(Component)]
pub struct Ent;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

// TODO: MAKE THIS METHOD ACTUALLY DO WHAT THE COMMENT BELOW SAYS
// This method applies velocity each tick to every entity
pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}
