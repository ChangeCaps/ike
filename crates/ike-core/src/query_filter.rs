use std::{any::TypeId, marker::PhantomData};

use crate::{AnyComponent, Entity, World};

pub trait QueryFilter {
	fn filter(world: &World, entity: &Entity) -> bool;
}

pub struct With<T>(PhantomData<fn() -> T>);

impl<T: AnyComponent> QueryFilter for With<T> {
	#[inline]
	fn filter(world: &World, entity: &Entity) -> bool {
		world.contains_component::<T>(entity)	
	}	
}

pub struct Without<T>(PhantomData<fn() -> T>);

impl<T: AnyComponent> QueryFilter for Without<T> {
	#[inline]
	fn filter(world: &World, entity: &Entity) -> bool {
		!world.contains_component::<T>(entity)	
	}
}

pub struct Changed<T>(PhantomData<fn() -> T>);

impl<T: AnyComponent> QueryFilter for Changed<T> {
	#[inline]
	fn filter(world: &World, entity: &Entity) -> bool {
		if let Some(storage) = world.components.get(&TypeId::of::<T>()) {
			storage.changed(entity, world.last_change_tick(), world.change_tick())
		} else {
			false
		}
	}
}

impl QueryFilter for () {
	#[inline]
	fn filter(_world: &World, _entity: &Entity) -> bool {
		true
	}
}

macro_rules! tuple_impl {
	($($name:ident),*) => {
		impl<$($name: QueryFilter),*> QueryFilter for ($($name,)*) {
			#[inline]
			fn filter(world: &World, entity: &Entity) -> bool {
				$($name::filter(world, entity))&&*
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

