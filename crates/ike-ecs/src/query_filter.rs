use crate::{ChangeTick, Entity, World};

pub trait QueryFilter {
    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool;
}

impl QueryFilter for () {
    fn filter(_world: &World, _entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        true
    }
}
