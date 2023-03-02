use crate::structs::{ent::Ent, ore_patch::OrePatch, structure::Structure, unit::Unit};

pub enum GameObject {
    Unit(Ent, Unit),
    Structure(Ent, Structure),
    OrePatch(Ent, OrePatch),
}
