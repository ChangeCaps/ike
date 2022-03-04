use std::{collections::BTreeSet, marker::PhantomData};

use crate::{
    Access, ChangeTick, ChangeTicks, Component, Entity, Mut, QueryIter, SystemAccess, World,
};

pub struct Query<'a, Q: WorldQuery> {
    world: &'a World,
    change_ticks: ChangeTicks,
    marker: PhantomData<fn() -> Q>,
}

impl<'a, Q: WorldQuery> Query<'a, Q> {
    pub fn new(world: &'a World, last_change_tick: ChangeTick) -> Option<Self> {
        if Q::Fetch::borrow(world) {
            Some(Self {
                world,
                change_ticks: ChangeTicks::new(last_change_tick, world.change_tick()),
                marker: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn iter(&self) -> QueryIter<'a, Q::ReadOnlyFetch> {
        unsafe { QueryIter::new(self.world, self.change_ticks.last_change_tick()) }
    }

    pub fn iter_mut(&mut self) -> QueryIter<'a, Q::Fetch> {
        unsafe { QueryIter::new(self.world, self.change_ticks.last_change_tick()) }
    }
}

impl<'a, Q: WorldQuery> Drop for Query<'a, Q> {
    fn drop(&mut self) {
        Q::Fetch::release(self.world);
    }
}

pub trait WorldQuery {
    type Fetch: for<'a> Fetch<'a>;
    type ReadOnlyFetch: for<'a> ReadOnlyFetch<'a>;
}

pub type QueryItem<'a, Q> = <<Q as WorldQuery>::Fetch as Fetch<'a>>::Item;

pub unsafe trait Fetch<'a>: Sized {
    type Item;
    type IterState: FetchIterState<'a>;

    fn access(access: &mut SystemAccess);

    fn borrow(_world: &World) -> bool {
        true
    }

    unsafe fn get(
        world: &'a World,
        entity: &Entity,
        change_ticks: &ChangeTicks,
    ) -> Option<Self::Item>;

    fn release(_world: &World) {}
}

pub unsafe trait ReadOnlyFetch<'a>: Fetch<'a> {}

pub trait FetchIterState<'a> {
    fn init(world: &'a World) -> Self;

    fn next_entity(&self, entity: &Entity) -> Option<Entity>;
}

impl WorldQuery for Entity {
    type Fetch = EntityFetch;
    type ReadOnlyFetch = EntityFetch;
}

pub struct EntityFetch;

pub struct EntityFetchIterState<'a> {
    entities: &'a BTreeSet<Entity>,
}

impl<'a> FetchIterState<'a> for EntityFetchIterState<'a> {
    fn init(world: &'a World) -> Self {
        Self {
            entities: world.entities().entities(),
        }
    }

    fn next_entity(&self, entity: &Entity) -> Option<Entity> {
        self.entities.range(entity..).next().cloned()
    }
}

unsafe impl<'a> Fetch<'a> for EntityFetch {
    type Item = Entity;
    type IterState = EntityFetchIterState<'a>;

    fn access(_access: &mut SystemAccess) {}

    unsafe fn get(
        _world: &'a World,
        entity: &Entity,
        _change_ticks: &ChangeTicks,
    ) -> Option<Self::Item> {
        Some(*entity)
    }
}

unsafe impl<'a> ReadOnlyFetch<'a> for EntityFetch {}

impl<'a, T: Component> WorldQuery for &'a T {
    type Fetch = FetchRead<T>;
    type ReadOnlyFetch = FetchRead<T>;
}

pub struct FetchRead<T>(PhantomData<fn() -> T>);

pub struct ComponentFetchIterState<'a, T> {
    entities: Option<&'a BTreeSet<Entity>>,
    marker: PhantomData<fn() -> T>,
}

impl<'a, T: Component> FetchIterState<'a> for ComponentFetchIterState<'a, T> {
    fn init(world: &'a World) -> Self {
        Self {
            entities: world.entities().storage().get_entities::<T>(),
            marker: PhantomData,
        }
    }

    fn next_entity(&self, entity: &Entity) -> Option<Entity> {
        self.entities?.range(entity..).next().cloned()
    }
}

unsafe impl<'a, T: Component> Fetch<'a> for FetchRead<T> {
    type Item = &'a T;
    type IterState = ComponentFetchIterState<'a, T>;

    fn access(access: &mut SystemAccess) {
        access.borrow_component::<T>(Access::Read);
    }

    fn borrow(world: &World) -> bool {
        world.entities().storage().borrow_storage::<T>()
    }

    unsafe fn get(world: &'a World, entity: &Entity, _: &ChangeTicks) -> Option<Self::Item> {
        let ptr = world.entities().storage().get_component_raw::<T>(entity)?;

        Some(unsafe { &*ptr })
    }

    fn release(world: &World) {
        world.entities().storage().release_storage::<T>();
    }
}

unsafe impl<'a, T: Component> ReadOnlyFetch<'a> for FetchRead<T> {}

impl<'a, T: Component> WorldQuery for &'a mut T {
    type Fetch = FetchWrite<T>;
    type ReadOnlyFetch = FetchReadOnlyWrite<T>;
}

pub struct FetchWrite<T>(PhantomData<fn() -> T>);
pub struct FetchReadOnlyWrite<T>(PhantomData<fn() -> T>);

unsafe impl<'a, T: Component> Fetch<'a> for FetchWrite<T> {
    type Item = Mut<'a, T>;
    type IterState = ComponentFetchIterState<'a, T>;

    fn access(access: &mut SystemAccess) {
        access.borrow_component::<T>(Access::Write);
    }

    fn borrow(world: &World) -> bool {
        world.entities().storage().borrow_storage_mut::<T>()
    }

    unsafe fn get(
        world: &'a World,
        entity: &Entity,
        _change_ticks: &ChangeTicks,
    ) -> Option<Self::Item> {
        let ptr = world.entities().storage().get_component_raw::<T>(entity)?;
        let data = unsafe { world.entities().storage().get_data_unchecked::<T>(entity) };

        Some(Mut::new(
            unsafe { &mut *ptr },
            &data.ticks.changed_raw(),
            world.change_tick(),
        ))
    }

    fn release(world: &World) {
        world.entities().storage().release_storage_mut::<T>();
    }
}

unsafe impl<'a, T: Component> Fetch<'a> for FetchReadOnlyWrite<T> {
    type Item = &'a T;
    type IterState = ComponentFetchIterState<'a, T>;

    fn access(access: &mut SystemAccess) {
        access.borrow_component::<T>(Access::Read);
    }

    fn borrow(world: &World) -> bool {
        world.entities().storage().borrow_storage::<T>()
    }

    unsafe fn get(
        world: &'a World,
        entity: &Entity,
        _change_ticks: &ChangeTicks,
    ) -> Option<Self::Item> {
        let ptr = world.entities().storage().get_component_raw::<T>(entity)?;

        Some(unsafe { &*ptr })
    }

    fn release(world: &World) {
        world.entities().storage().release_storage::<T>();
    }
}

unsafe impl<'a, T: Component> ReadOnlyFetch<'a> for FetchReadOnlyWrite<T> {}

macro_rules! tuple_world_query {
    () => {};
    ($first:ident $(,$name:ident)* $(,)?) => {
        tuple_world_query!(@ $first, $($name),*);
        tuple_world_query!($($name),*);
    };
    (@ $($name:ident),* $(,)?) => {
        #[allow(non_snake_case)]
        impl<'a, $($name: FetchIterState<'a>),*> FetchIterState<'a> for ($($name,)*) {
            fn init(world: &'a World) -> Self {
                ($($name::init(world),)*)
            }

            fn next_entity(&self, entity: &Entity) -> Option<Entity> {
                let ($($name,)*) = self;

                [$($name.next_entity(entity)?),*].into_iter().min()
            }
        }

        #[allow(non_snake_case)]
        unsafe impl<'a, $($name: Fetch<'a>),*> Fetch<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);
            type IterState = ($($name::IterState,)*);

            fn access(access: &mut SystemAccess) {
                $($name::access(access);)*
            }

            fn borrow(world: &World) -> bool {
                let ($($name,)*) = ($($name::borrow(world),)*);

                let borrowed = $($name)&&*;

                if !borrowed {
                    $(
                        if !$name {
                            $name::release(world);
                        }
                    )*
                }

                borrowed
            }

            unsafe fn get(world: &'a World, entity: &Entity, change_ticks: &ChangeTicks) -> Option<Self::Item> {
                Some(unsafe {($($name::get(world, entity, change_ticks)?,)*)})
            }

            fn release(world: &World) {
                $($name::release(world);)*
            }
        }

        unsafe impl<'a, $($name: ReadOnlyFetch<'a>),*> ReadOnlyFetch<'a> for ($($name,)*) {}

        impl<$($name: WorldQuery),*> WorldQuery for ($($name,)*) {
            type Fetch = ($($name::Fetch,)*);
            type ReadOnlyFetch = ($($name::ReadOnlyFetch,)*);
        }
    };
}

tuple_world_query!(A, B, C, D, E, F, G, H, I, J, L, M, N, O, P);
