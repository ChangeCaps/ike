use std::marker::PhantomData;

use crate::{ChangeTick, ChangeTicks, Component, Entity, World};

pub trait QueryFilter {
    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool;
}

impl QueryFilter for () {
    fn filter(_world: &World, _entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        true
    }
}

pub struct Changed<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for Changed<T> {
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
    fn filter(world: &World, entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        world.entities().contains_component::<T>(entity)
    }
}

pub struct Without<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for Without<T> {
    fn filter(world: &World, entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        !world.entities().contains_component::<T>(entity)
    }
}

macro_rules! impl_query_filter {
    () => {};
    ($first:ident $(,$name:ident)*) => {
        impl_query_filter!($($name),*);
        impl_query_filter!(@ $first $(,$name)*);
    };
    (@ $($name:ident),*) => {
        impl<$($name: QueryFilter),*> QueryFilter for ($($name,)*) {
            fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool {
                $($name::filter(world, entity, last_change_tick))||*
            }
        }
    };
}

impl_query_filter!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
