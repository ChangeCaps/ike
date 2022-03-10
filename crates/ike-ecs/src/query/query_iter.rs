use std::{borrow::Cow, marker::PhantomData};

use crate::{
    ChangeTick, ChangeTicks, Entity, EntitySet, EntitySetIntoIter, EntitySetIter, Fetch,
    QueryFilter, World,
};

enum EntitiesIter<'a> {
    Borrowed(EntitySetIter<'a>),
    Owned(EntitySetIntoIter),
}

impl<'a> Iterator for EntitiesIter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Borrowed(iter) => iter.next(),
            Self::Owned(iter) => iter.next(),
        }
    }
}

impl<'a> From<Cow<'a, EntitySet>> for EntitiesIter<'a> {
    fn from(entity_set: Cow<'a, EntitySet>) -> Self {
        match entity_set {
            Cow::Borrowed(set) => Self::Borrowed(set.iter()),
            Cow::Owned(set) => Self::Owned(set.into_iter()),
        }
    }
}

pub struct QueryIter<'a, F: Fetch<'a>, QF: QueryFilter = ()> {
    entities: EntitiesIter<'a>,
    world: &'a World,
    change_ticks: ChangeTicks,
    marker: PhantomData<fn() -> (F, QF)>,
}

impl<'a, F: Fetch<'a>, QF: QueryFilter> QueryIter<'a, F, QF> {
    /// # Safety
    /// - must not be able to break borrow rules for any components.
    pub unsafe fn new(world: &'a World, last_change_tick: ChangeTick) -> Self {
        let mut entities = F::entities(&world);

        match QF::entities(world) {
            Some(filter) => {
                let mut owned = entities.into_owned();

                owned.and(&filter);

                entities = Cow::Owned(owned);
            }
            None => {}
        }

        Self {
            entities: entities.into(),
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
            let entity = self.entities.next()?;

            if QF::filter(self.world, &entity, self.change_ticks.last_change_tick()) {
                let item = unsafe { F::get(self.world, &entity, &self.change_ticks) };

                if item.is_some() {
                    break item;
                }
            }
        }
    }
}
