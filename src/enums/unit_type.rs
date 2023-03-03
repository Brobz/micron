use crate::structs::unit::Unit;

pub enum UnitType {
    Scout(Unit),
    Miner(Unit),
    Collector(Unit),
}
