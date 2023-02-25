use std::collections::HashMap;

use vector2d::Vector2D;

use super::ent::{Ent, EntID};

pub struct WorldInfo {
    ent_hp: HashMap<EntID, f32>,                 // Stores entity hp
    ent_position: HashMap<EntID, Vector2D<f32>>, // Stores entity rect center
    ent_rect: HashMap<EntID, (u32, u32)>,        // Stores entity rect dimensions
}

impl WorldInfo {
    pub fn new() -> WorldInfo {
        WorldInfo {
            ent_hp: HashMap::new(),
            ent_position: HashMap::new(),
            ent_rect: HashMap::new(),
        }
    }

    pub fn _get_ent_poisition(&self, ent: &Ent) -> Option<Vector2D<f32>> {
        return self.ent_position.get(&ent.id).copied();
    }

    pub fn get_ent_hp(&self, ent: &Ent) -> Option<f32> {
        return self.ent_hp.get(&ent.id).copied();
    }

    pub fn get_ent_poisition_by_id(&self, ent_id: &EntID) -> Option<Vector2D<f32>> {
        return self.ent_position.get(ent_id).copied();
    }

    pub fn update_ent(&mut self, ent: &Ent) {
        self.remove_ent(ent);

        let ent_rect = ent.get_rect();
        let ent_rect_center = &ent_rect.center();
        self.ent_hp.insert(ent.id, ent.hp);
        self.ent_position.insert(
            ent.id,
            Vector2D::new(ent_rect_center.x as f32, ent_rect_center.y as f32),
        );
        self.ent_rect.insert(ent.id, ent_rect.size());
    }

    pub fn damage_ent(&mut self, ent_id: &EntID, dmg: u32) {
        if self.ent_hp.get_mut(&ent_id).is_some() {
            let new_hp = *self.ent_hp.get_mut(&ent_id).unwrap() - dmg as f32;
            if new_hp < 0.0 {
                self.remove_ent_by_id(ent_id);
            } else {
                self.ent_hp.remove(&ent_id);
                self.ent_hp.insert(*ent_id, new_hp);
            }
        }
    }

    pub fn add_ent(&mut self, ent: &Ent) {
        let ent_rect = ent.get_rect();
        let ent_rect_center = &ent_rect.center();
        self.ent_hp.insert(ent.id, ent.hp);
        self.ent_position.insert(
            ent.id,
            Vector2D::new(ent_rect_center.x as f32, ent_rect_center.y as f32),
        );
        self.ent_rect.insert(ent.id, ent_rect.size());
    }

    pub fn remove_ent(&mut self, ent: &Ent) {
        if self.ent_position.get_mut(&ent.id).is_some() {
            self.ent_position.remove(&ent.id);
        }
        if self.ent_rect.get_mut(&ent.id).is_some() {
            self.ent_rect.remove(&ent.id);
        }
        if self.ent_hp.get_mut(&ent.id).is_some() {
            self.ent_hp.remove(&ent.id);
        }
    }

    pub fn remove_ent_by_id(&mut self, ent_id: &EntID) {
        if self.ent_position.get_mut(&ent_id).is_some() {
            self.ent_position.remove(&ent_id);
        }
        if self.ent_rect.get_mut(&ent_id).is_some() {
            self.ent_rect.remove(&ent_id);
        }
        if self.ent_hp.get_mut(&ent_id).is_some() {
            self.ent_hp.remove(&ent_id);
        }
    }

    pub fn has_ent(&self, ent: &Ent) -> bool {
        if self.ent_hp.get(&ent.id).is_some() {
            return true;
        }
        false
    }

    pub fn has_ent_by_id(&self, ent_id: &EntID) -> bool {
        if self.ent_hp.get(&ent_id).is_some() {
            return true;
        }
        false
    }
}
