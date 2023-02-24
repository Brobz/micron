use std::collections::HashMap;

use vector2d::Vector2D;

use super::ent::{Ent, EntID};

pub struct WorldInfo {
    ent_position: HashMap<EntID, Vector2D<f32>>, // Stores entity rect center
    ent_rect: HashMap<EntID, (u32, u32)>,        // Stores entity rect dimensions
}

impl WorldInfo {
    pub fn new() -> WorldInfo {
        WorldInfo {
            ent_position: HashMap::new(),
            ent_rect: HashMap::new(),
        }
    }

    pub fn _get_ent_poisition(&self, ent: &Ent) -> Option<Vector2D<f32>> {
        return self.ent_position.get(&ent.id).copied();
    }

    pub fn get_ent_poisition_by_id(&self, ent_id: &EntID) -> Option<Vector2D<f32>> {
        return self.ent_position.get(ent_id).copied();
    }

    pub fn update_ent(&mut self, ent: &Ent) {
        if self.ent_position.get_mut(&ent.id).is_some() {
            self.ent_position.remove(&ent.id);
        }
        if self.ent_rect.get_mut(&ent.id).is_some() {
            self.ent_rect.remove(&ent.id);
        }

        let ent_rect = ent.get_rect();
        let ent_rect_center = &ent_rect.center();
        self.ent_position.insert(
            ent.id,
            Vector2D::new(ent_rect_center.x as f32, ent_rect_center.y as f32),
        );
        self.ent_rect.insert(ent.id, ent_rect.size());
    }

    pub fn add_ent(&mut self, ent: &Ent) {
        let ent_rect = ent.get_rect();
        let ent_rect_center = &ent_rect.center();
        self.ent_position.insert(
            ent.id,
            Vector2D::new(ent_rect_center.x as f32, ent_rect_center.y as f32),
        );
        self.ent_rect.insert(ent.id, ent_rect.size());
    }
}
