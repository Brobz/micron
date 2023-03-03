use crate::structs::{ent::Ent, ore::Ore, ore_patch::OrePatch, structure::Structure};

use super::unit_type::UnitType;

pub enum GameObject {
    Unit(Ent, UnitType),
    Structure(Ent, Structure),
    OrePatch(Ent, OrePatch),
    Ore(Ent, Ore),
}
