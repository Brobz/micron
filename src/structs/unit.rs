use bevy::prelude::*;

use crate::Collider;
use crate::Velocity;

use crate::MouseInfo;
use crate::Selection;

#[derive(Component)]
pub struct Unit;

// This method returns a normalized vector with size speed that points from, to
pub fn get_walk_velocity_to(from: Vec2, to: Vec2, speed: f32) -> Vec2 {
    return (to - from).normalize() * Vec2::new(speed, speed);
}

// This method issues commands to the current selection of units
pub fn issue_commands(
    mouse_info: Res<MouseInfo>,
    mut selection: ResMut<Selection>,
    mut query: Query<(Entity, &Transform, &mut Velocity, Option<&Unit>), With<Collider>>,
) {
    if mouse_info.right_button {
        // issue move command to mouse position
        for selected_entity in &mut selection.current {
            for (entity, transform, mut vel, unit_flag) in &mut query {
                if entity == *selected_entity && unit_flag.is_some() {
                    *vel = Velocity(get_walk_velocity_to(
                        Vec2::new(transform.translation.x, transform.translation.y),
                        mouse_info.position,
                        100.0,
                    ));
                }
            }
        }
        // selection.set_curr(Vec::<Entity>::new()) // this clears selection
    }
}
