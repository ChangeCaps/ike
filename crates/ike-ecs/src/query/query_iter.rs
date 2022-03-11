use std::marker::PhantomData;

use crate::{ChangeTick, ChangeTicks, EntitySet, EntitySetIter, Fetch, QueryFilter, World};

pub struct QueryIter<'a, F: Fetch<'a>, QF: QueryFilter = ()> {
    entities: Option<EntitySetIter<'a>>,
    world: &'a World,
    change_ticks: ChangeTicks,
    marker: PhantomData<fn() -> (F, QF)>,
}

impl<'a, F: Fetch<'a>, QF: QueryFilter> QueryIter<'a, F, QF> {
    /// # Safety
    /// - must not be able to break borrow rules for any components.
    pub unsafe fn new(world: &'a World, last_change_tick: ChangeTick) -> Self {
        #[cfg(feature = "trace")]
        let fetch_entities_span = ike_util::tracing::info_span!("query entities fetch");
        #[cfg(feature = "trace")]
        let fetch_entities_guard = fetch_entities_span.enter();

        let mut entities = F::entities(&world);

        #[cfg(feature = "trace")]
        drop(fetch_entities_guard);

        #[cfg(feature = "trace")]
        let filter_entities_span = ike_util::tracing::info_span!("query filter fetch");
        #[cfg(feature = "trace")]
        let filter_entities_guard = filter_entities_span.enter();

        match QF::entities(world) {
            Some(filter) => {
                if let Some(ref mut set) = entities {
                    if filter.len() < set.len() {
                        *set = filter;
                    }
                }
            }
            None => {}
        }

        #[cfg(feature = "trace")]
        drop(filter_entities_guard);

        Self {
            entities: entities.map(EntitySet::iter),
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
            let entity = self.entities.as_mut()?.next()?;

            if QF::filter(self.world, &entity, self.change_ticks.last_change_tick()) {
                let item = unsafe { F::get(self.world, &entity, &self.change_ticks) };

                if item.is_some() {
                    break item;
                }
            }
        }
    }
}
