use bevy::prelude::*;

#[derive(Component)]
pub struct Collider;

#[derive(Default)]
pub struct CollisionEvent;

#[derive(Resource)]
pub struct CollisionSound(pub Handle<AudioSource>);
