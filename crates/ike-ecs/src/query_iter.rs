use crate::{ChangeTick, ChangeTicks, Entity, Fetch, FetchIterState, World};

pub struct QueryIter<'a, F: Fetch<'a>> {
    state: F::IterState,
    entity: Option<Entity>,
    world: &'a World,
    change_ticks: ChangeTicks,
}

impl<'a, F: Fetch<'a>> QueryIter<'a, F> {
    pub unsafe fn new(world: &'a World, last_change_tick: ChangeTick) -> Self {
        let state = F::IterState::init(world);

        let entity = world.entities().entities().iter().next().cloned();

        Self {
            state,
            entity,
            world,
            change_ticks: ChangeTicks::new(last_change_tick, world.change_tick()),
        }
    }
}

impl<'a, F: Fetch<'a>> Iterator for QueryIter<'a, F> {
    type Item = F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let entity = self.state.next_entity(self.entity.as_ref()?)?;

        unsafe { F::get(self.world, &entity, &self.change_ticks) }
    }
}
