use std::marker::PhantomData;

use crate::{AnyComponent, Entity, World};

pub trait QueryFilter {
    fn filter(world: &World, entity: &Entity, last_change_tick: u64) -> bool;
}

pub struct With<T>(PhantomData<fn() -> T>);

impl<T: AnyComponent> QueryFilter for With<T> {
    #[inline]
    fn filter(world: &World, entity: &Entity, _last_change_tick: u64) -> bool {
        world.entities().contains::<T>(entity)
    }
}

pub struct Without<T>(PhantomData<fn() -> T>);

impl<T: AnyComponent> QueryFilter for Without<T> {
    #[inline]
    fn filter(world: &World, entity: &Entity, _last_change_tick: u64) -> bool {
        !world.entities().contains::<T>(entity)
    }
}

pub struct Changed<T>(PhantomData<fn() -> T>);

impl<T: AnyComponent> QueryFilter for Changed<T> {
    #[inline]
    fn filter(world: &World, entity: &Entity, last_change_tick: u64) -> bool {
        if let Some(storage) = world.entities().storage::<T>() {
            storage.changed(entity, last_change_tick, world.change_tick())
        } else {
            false
        }
    }
}

pub struct Or<T, U>(PhantomData<fn() -> (T, U)>);

impl<T: QueryFilter, U: QueryFilter> QueryFilter for Or<T, U> {
    #[inline]
    fn filter(world: &World, entity: &Entity, last_change_tick: u64) -> bool {
        T::filter(world, entity, last_change_tick) || U::filter(world, entity, last_change_tick)
    }
}

impl QueryFilter for () {
    #[inline]
    fn filter(_world: &World, _entity: &Entity, _last_change_tick: u64) -> bool {
        true
    }
}

macro_rules! tuple_impl {
	($($name:ident),*) => {
		impl<$($name: QueryFilter),*> QueryFilter for ($($name,)*) {
			#[inline]
			fn filter(world: &World, entity: &Entity, last_change_tick: u64) -> bool {
				$($name::filter(world, entity, last_change_tick))&&*
			}
		}
	};
}

macro_rules! tuples {
	($macro:ident, $name:ident, $($names:ident),*) => {
		$macro!($name, $($names),*);
		tuples!($macro, $($names),*);
	};
	($macro:ident, $name:ident) => {
		$macro!($name);
	};
}

tuples!(tuple_impl, A, B, C, D, E, F, G, H, I, J, K);
