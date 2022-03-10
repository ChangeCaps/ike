use std::{borrow::Cow, marker::PhantomData};

use crate::{ChangeTick, ChangeTicks, Component, Entity, EntitySet, World};

pub trait QueryFilter {
    fn entities(world: &World) -> Option<Cow<EntitySet>>;

    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool;
}

impl QueryFilter for () {
    fn entities(_world: &World) -> Option<Cow<EntitySet>> {
        None
    }

    fn filter(_world: &World, _entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        true
    }
}

pub struct Changed<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for Changed<T> {
    fn entities(world: &World) -> Option<Cow<EntitySet>> {
        Some(Cow::Borrowed(
            world.entities().storage().get_entities::<T>()?,
        ))
    }

    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool {
        if let Some(ticks) = world.entities().storage().get_component_ticks::<T>(entity) {
            ticks.is_changed(&ChangeTicks::new(last_change_tick, world.change_tick()))
        } else {
            false
        }
    }
}

pub struct Added<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for Added<T> {
    fn entities(world: &World) -> Option<Cow<EntitySet>> {
        Some(Cow::Borrowed(
            world.entities().storage().get_entities::<T>()?,
        ))
    }

    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool {
        if let Some(ticks) = world.entities().storage().get_component_ticks::<T>(entity) {
            ticks.is_added(&ChangeTicks::new(last_change_tick, world.change_tick()))
        } else {
            false
        }
    }
}

pub struct With<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for With<T> {
    fn entities(world: &World) -> Option<Cow<EntitySet>> {
        Some(Cow::Borrowed(
            world.entities().storage().get_entities::<T>()?,
        ))
    }

    fn filter(world: &World, entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        world.entities().contains_component::<T>(entity)
    }
}

pub struct Without<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for Without<T> {
    fn entities(world: &World) -> Option<Cow<EntitySet>> {
        let with = world.entities().storage().get_entities::<T>()?;
        let mut entities = world.entities().entities().clone();
        entities.nand(with);

        Some(Cow::Owned(entities))
    }

    fn filter(world: &World, entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        !world.entities().contains_component::<T>(entity)
    }
}

pub struct Or<T, U>(PhantomData<fn() -> (T, U)>);

impl<T: QueryFilter, U: QueryFilter> QueryFilter for Or<T, U> {
    fn entities(world: &World) -> Option<Cow<EntitySet>> {
        let mut left = T::entities(world)?.into_owned();
        let right = U::entities(world)?;
        left.or(&right);
        Some(Cow::Owned(left))
    }

    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool {
        T::filter(world, entity, last_change_tick) || U::filter(world, entity, last_change_tick)
    }
}

macro_rules! impl_query_filter {
    () => {};
    ($first:ident $(,$name:ident)*) => {
        impl_query_filter!($($name),*);
        impl_query_filter!(@ $first $(,$name)*);
    };
    (@entities $first:ident $(,$name:ident)+) => {
        let mut entities = $first.into_owned();

        $(
            entities.and(&$name);
        )
        *

        return Some(Cow::Owned(entities));
    };
    (@entities $first:ident) => {
        return Some($first);
    };
    (@ $($name:ident),*) => {
        impl<$($name: QueryFilter),*> QueryFilter for ($($name,)*) {
            #[allow(non_snake_case)]
            fn entities(world: &World) -> Option<Cow<EntitySet>> {
                $(
                    let $name = $name::entities(world)?;
                )
                *

                impl_query_filter!(@entities $($name),*);
            }


            fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool {
                $($name::filter(world, entity, last_change_tick))&&*
            }
        }
    };
}

impl_query_filter!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
