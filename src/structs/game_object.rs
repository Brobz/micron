use super::{ent::Ent, structure::Structure, unit::Unit};

// The idea is:
// 1. Everywhere in the code, try to ALWAYS use the outter ent for stuff
// 2. Inside of U / S, everything that would take E now takes a parameter for E (E will not be stored inside of U / S anymore) {have to try this}

pub enum GameObject {
    Unit(Ent, Unit),
    Structure(Ent, Structure),
}
