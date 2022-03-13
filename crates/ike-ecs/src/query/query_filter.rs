use std::marker::PhantomData;

use crate::{ChangeTick, ChangeTicks, Component, Entity, EntitySet, TypeRegistry, World};

pub trait QueryFilter {
    fn entities(world: &World) -> Option<&EntitySet>;

    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool;

    fn register_types(type_registry: &mut TypeRegistry);
}

impl QueryFilter for () {
    fn entities(_world: &World) -> Option<&EntitySet> {
        None
    }

    fn filter(_world: &World, _entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        true
    }

    fn register_types(_type_registry: &mut TypeRegistry) {}
}

pub struct Changed<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for Changed<T> {
    fn entities(world: &World) -> Option<&EntitySet> {
        world.entities().storage().get_entities::<T>()
    }

    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool {
        if let Some(ticks) = world.entities().storage().get_component_ticks::<T>(entity) {
            ticks.is_changed(&ChangeTicks::new(last_change_tick, world.change_tick()))
        } else {
            false
        }
    }

    fn register_types(type_registry: &mut TypeRegistry) {
        type_registry.insert_registration::<T>(T::type_registration());
    }
}

pub struct Added<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for Added<T> {
    fn entities(world: &World) -> Option<&EntitySet> {
        world.entities().storage().get_entities::<T>()
    }

    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool {
        if let Some(ticks) = world.entities().storage().get_component_ticks::<T>(entity) {
            ticks.is_added(&ChangeTicks::new(last_change_tick, world.change_tick()))
        } else {
            false
        }
    }

    fn register_types(type_registry: &mut TypeRegistry) {
        type_registry.insert_registration::<T>(T::type_registration());
    }
}

pub struct With<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for With<T> {
    fn entities(world: &World) -> Option<&EntitySet> {
        world.entities().storage().get_entities::<T>()
    }

    fn filter(world: &World, entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        world.entities().contains_component::<T>(entity)
    }

    fn register_types(type_registry: &mut TypeRegistry) {
        type_registry.insert_registration::<T>(T::type_registration());
    }
}

pub struct Without<T>(PhantomData<fn() -> T>);

impl<T: Component> QueryFilter for Without<T> {
    fn entities(world: &World) -> Option<&EntitySet> {
        Some(world.entities().entities())
    }

    fn filter(world: &World, entity: &Entity, _last_change_tick: ChangeTick) -> bool {
        !world.entities().contains_component::<T>(entity)
    }

    fn register_types(type_registry: &mut TypeRegistry) {
        type_registry.insert_registration::<T>(T::type_registration());
    }
}

pub struct Or<T, U>(PhantomData<fn() -> (T, U)>);

impl<T: QueryFilter, U: QueryFilter> QueryFilter for Or<T, U> {
    fn entities(world: &World) -> Option<&EntitySet> {
        if let Some(set) = T::entities(world) {
            if let Some(other) = U::entities(world) {
                if set.len() < other.len() {
                    Some(set)
                } else {
                    Some(other)
                }
            } else {
                Some(set)
            }
        } else {
            U::entities(world)
        }
    }

    fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool {
        T::filter(world, entity, last_change_tick) || U::filter(world, entity, last_change_tick)
    }

    fn register_types(type_registry: &mut TypeRegistry) {
        T::register_types(type_registry);
        U::register_types(type_registry);
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
            #[allow(non_snake_case)]
            fn entities(world: &World) -> Option<&EntitySet> {
                [$($name::entities(world)?),*].into_iter().min_by(|a, b| a.len().cmp(&b.len()))
            }


            fn filter(world: &World, entity: &Entity, last_change_tick: ChangeTick) -> bool {
                $($name::filter(world, entity, last_change_tick))&&*
            }

            fn register_types(type_registry: &mut TypeRegistry) {
                $($name::register_types(type_registry);)*
            }
        }
    };
}

impl_query_filter!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
