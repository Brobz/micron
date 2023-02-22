use bevy::prelude::*;

use crate::collide;

use crate::consts::setup::SELECTION_BOX_COLOR;
use crate::Collider;
use crate::CollisionEvent;
use crate::MouseInfo;
use crate::Unit;

// This resource tags the selection box entity that gets drawn on the screen
#[derive(Component)]
pub struct SelectionBox;

// This resource tracks the current selection of units and structures
#[derive(Resource)]
pub struct Selection {
    pub open: bool,
    pub just_closed: bool,
    pub origin: Vec2,
    pub center: Vec3,
    pub scale: Vec3,
    pub current: Vec<Entity>,
}

impl Selection {
    pub fn set_curr(&mut self, _selection: Vec<Entity>) {
        println!("{}", _selection.len().to_string());
        self.current = _selection
    }
}

// This method gets and stores cursor position
pub fn get_cursor_position(mut mouse_info: ResMut<MouseInfo>, windows: Res<Windows>) {
    // Games typically only have one window (the primary window).
    // For multi-window applications, you need to use a specific window ID here.
    let window = windows.get_primary().unwrap();

    if let Some(_position) = window.cursor_position() {
        mouse_info.set_pos(_position);
    } else {
        // cursor is not inside the window
        mouse_info.set_pos(Vec2::from([-1f32, -1f32]));
    }
}

// This method gets and stores mouse button input
pub fn mouse_button_input(mut mouse_info: ResMut<MouseInfo>, buttons: Res<Input<MouseButton>>) {
    mouse_info.left_button = buttons.pressed(MouseButton::Left);
    mouse_info.right_button = buttons.pressed(MouseButton::Right);
}

// This method takes in a vec2 position of where the mouse is currently, and one of where it was originally clicked;
// Then it returns a vec2 which pertains to the center of a square where each one of the two inputs represent opposing corners
pub fn find_selection_box_translation(curr_pos: Vec2, origin: Vec2) -> Vec3 {
    let x: f32 = (curr_pos.x + origin.x) / 2.0;
    let y: f32 = (curr_pos.y + origin.y) / 2.0;
    return Vec3::new(x, y, 1.0);
}

// This method calculates and draws the selection box
pub fn draw_selection_box(
    mouse_info: Res<MouseInfo>,
    mut selection: ResMut<Selection>,
    mut commands: Commands,
    mut transform_query: Query<&mut Transform, With<SelectionBox>>,
    entity_query: Query<Entity, With<SelectionBox>>,
) {
    if mouse_info.left_button {
        if selection.open {
            let mut selection_box_transform = transform_query.single_mut();
            selection_box_transform.translation =
                find_selection_box_translation(mouse_info.position, selection.origin);
            selection_box_transform.scale = Vec3::new(
                (mouse_info.position.x - selection.origin.x).abs(),
                (mouse_info.position.y - selection.origin.y).abs(),
                1.0,
            );
            selection.center = selection_box_transform.translation;
            selection.scale = selection_box_transform.scale;
        } else {
            selection.open = true;
            selection.origin = Vec2::new(mouse_info.position.x, mouse_info.position.y);
            commands.spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: find_selection_box_translation(
                            mouse_info.position,
                            selection.origin,
                        ),
                        scale: Vec3::new(
                            (mouse_info.position.x - selection.origin.x).abs(),
                            (mouse_info.position.y - selection.origin.y).abs(),
                            1.0,
                        ),
                        ..default()
                    },
                    sprite: Sprite {
                        color: SELECTION_BOX_COLOR,
                        ..default()
                    },
                    ..default()
                },
                SelectionBox,
                Collider,
            ));
        }
    } else {
        if selection.open {
            selection.just_closed = true;
            selection.open = false;
            commands.entity(entity_query.single()).despawn();
        }
    }
}

// This method checks the drawn collision box aggainst everything that has collider
// then adds any collisions into the current selection, if needed (selection just issued)
pub fn check_for_selection_box_collisions(
    mut selection: ResMut<Selection>,
    collider_query: Query<(Entity, &Transform, Option<&Unit>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let selection_size = selection.scale.truncate();
    let mut final_selection = Vec::<Entity>::new();

    for (collider_entity, transform, unit_flag) in &collider_query {
        let collision = collide(
            selection.center,
            selection_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(_collision) = collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send_default();

            if unit_flag.is_some() {
                final_selection.push(collider_entity)
            }
        }
    }

    if selection.just_closed {
        selection.set_curr(final_selection);
        selection.just_closed = false;
    }
}
