use std::marker::PhantomData;

use crate::{ChangeTick, ChangeTicks, Entity, Fetch, FetchIterState, QueryFilter, World};

pub struct QueryIter<'a, F: Fetch<'a>, QF: QueryFilter = ()> {
    state: F::IterState,
    entity: Option<Entity>,
    world: &'a World,
    change_ticks: ChangeTicks,
    marker: PhantomData<fn() -> QF>,
}

impl<'a, F: Fetch<'a>, QF: QueryFilter> QueryIter<'a, F, QF> {
    /// # Safety
    /// - must not be able to break borrow rules for any components.
    pub unsafe fn new(world: &'a World, last_change_tick: ChangeTick) -> Self {
        let state = F::IterState::init(world);

        let entity = world.entities().entities().first();

        Self {
            state,
            entity,
            world,
            change_ticks: ChangeTicks::new(last_change_tick, world.change_tick()),
            marker: PhantomData,
        }
    }
}

impl<'a, F: Fetch<'a>, QF: QueryFilter> Iterator for QueryIter<'a, F, QF> {
    type Item = F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entity = self.entity.take()?;
            self.entity = self.state.next_entity(&entity);

            if QF::filter(self.world, &entity, self.change_ticks.last_change_tick()) {
                let item = unsafe { F::get(self.world, &entity, &self.change_ticks) };

                if item.is_some() {
                    break item;
                }
            }
        }
    }
}
