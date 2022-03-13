use std::marker::PhantomData;

use ike_task::TaskPool;

use crate::{
    Access, ChangeTick, ChangeTicks, Component, Entity, EntitySet, Mut, QueryFilter, QueryIter,
    SystemAccess, TypeRegistry, World,
};

pub struct Query<'a, Q: WorldQuery, F: QueryFilter = ()> {
    world: &'a World,
    change_ticks: ChangeTicks,
    marker: PhantomData<fn() -> (Q, F)>,
}

impl<'a, Q: WorldQuery, F: QueryFilter> Query<'a, Q, F> {
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

    pub fn iter(&self) -> QueryIter<'a, Q::ReadOnlyFetch, F> {
        // SAFETY:
        // creating a Query requires ensuring component access is valid
        unsafe { QueryIter::new(self.world, self.change_ticks.last_change_tick()) }
    }

    pub fn iter_mut(&mut self) -> QueryIter<'a, Q::Fetch, F> {
        // SAFETY:
        // creating a Query requires ensuring component access is valid
        unsafe { QueryIter::new(self.world, self.change_ticks.last_change_tick()) }
    }

    pub fn for_each(&self, mut f: impl FnMut(<Q::ReadOnlyFetch as Fetch>::Item)) {
        for item in self.iter() {
            f(item);
        }
    }

    pub fn for_each_mut(&mut self, mut f: impl FnMut(<Q::Fetch as Fetch>::Item)) {
        for item in self.iter_mut() {
            f(item);
        }
    }

    pub fn par_for_each(
        &self,
        task_pool: &TaskPool,
        f: impl Fn(<Q::ReadOnlyFetch as Fetch>::Item) + Send + Sync,
    ) where
        <Q::ReadOnlyFetch as Fetch<'a>>::Item: Send,
    {
        task_pool.scope(|scope| {
            for item in self.iter() {
                scope.spawn(async {
                    f(item);
                });
            }
        });
    }

    pub fn par_for_each_mut(
        &mut self,
        task_pool: &TaskPool,
        f: impl Fn(<Q::Fetch as Fetch>::Item) + Send + Sync,
    ) where
        <Q::Fetch as Fetch<'a>>::Item: Send,
    {
        task_pool.scope(|scope| {
            for item in self.iter_mut() {
                scope.spawn(async {
                    f(item);
                });
            }
        });
    }

    pub fn contains(&self, entity: &Entity) -> bool {
        self.get(entity).is_some()
    }

    pub fn get(&self, entity: &Entity) -> Option<<Q::ReadOnlyFetch as Fetch<'a>>::Item> {
        if F::filter(self.world, entity, self.change_ticks.last_change_tick()) {
            // SAFETY:
            // creating a Query requires ensuring component access is valid
            unsafe { Q::ReadOnlyFetch::get(self.world, entity, &self.change_ticks) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity: &Entity) -> Option<QueryItem<'a, Q>> {
        if F::filter(self.world, entity, self.change_ticks.last_change_tick()) {
            // SAFETY:
            // creating a Query requires ensuring component access is valid
            unsafe { Q::Fetch::get(self.world, entity, &self.change_ticks) }
        } else {
            None
        }
    }
}

impl<'a, Q: WorldQuery, F: QueryFilter> Drop for Query<'a, Q, F> {
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

    fn access(access: &mut SystemAccess);

    fn borrow(_world: &World) -> bool {
        true
    }

    fn entities(world: &'a World) -> Option<&'a EntitySet>;

    unsafe fn get(
        world: &'a World,
        entity: &Entity,
        change_ticks: &ChangeTicks,
    ) -> Option<Self::Item>;

    fn release(_world: &World) {}

    fn register_types(type_registry: &mut TypeRegistry);
}

pub unsafe trait ReadOnlyFetch<'a>: Fetch<'a> {}

impl WorldQuery for Entity {
    type Fetch = EntityFetch;
    type ReadOnlyFetch = EntityFetch;
}

pub struct EntityFetch;

unsafe impl<'a> Fetch<'a> for EntityFetch {
    type Item = Entity;

    fn access(_access: &mut SystemAccess) {}

    fn entities(world: &'a World) -> Option<&'a EntitySet> {
        Some(world.entities().entities())
    }

    unsafe fn get(
        world: &'a World,
        entity: &Entity,
        _change_ticks: &ChangeTicks,
    ) -> Option<Self::Item> {
        if world.entities().entities().contains(entity) {
            Some(*entity)
        } else {
            None
        }
    }

    fn register_types(_type_registry: &mut TypeRegistry) {}
}

unsafe impl<'a> ReadOnlyFetch<'a> for EntityFetch {}

impl<'a, T: Component> WorldQuery for &'a T {
    type Fetch = FetchRead<T>;
    type ReadOnlyFetch = FetchRead<T>;
}

pub struct FetchRead<T>(PhantomData<fn() -> T>);

unsafe impl<'a, T: Component> Fetch<'a> for FetchRead<T> {
    type Item = &'a T;

    fn access(access: &mut SystemAccess) {
        access.borrow_component::<T>(Access::Read);
    }

    fn entities(world: &'a World) -> Option<&EntitySet> {
        world.entities().storage().get_entities::<T>()
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

    fn register_types(type_registry: &mut TypeRegistry) {
        type_registry.insert_registration::<T>(T::type_registration());
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

    fn access(access: &mut SystemAccess) {
        access.borrow_component::<T>(Access::Write);
    }

    fn entities(world: &'a World) -> Option<&'a EntitySet> {
        world.entities().storage().get_entities::<T>()
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

    fn register_types(type_registry: &mut TypeRegistry) {
        type_registry.insert_registration::<T>(T::type_registration());
    }
}

unsafe impl<'a, T: Component> Fetch<'a> for FetchReadOnlyWrite<T> {
    type Item = &'a T;

    fn access(access: &mut SystemAccess) {
        access.borrow_component::<T>(Access::Read);
    }

    fn entities(world: &'a World) -> Option<&'a EntitySet> {
        world.entities().storage().get_entities::<T>()
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

    fn register_types(type_registry: &mut TypeRegistry) {
        type_registry.insert_registration::<T>(T::type_registration());
    }
}

unsafe impl<'a, T: Component> ReadOnlyFetch<'a> for FetchReadOnlyWrite<T> {}

impl<T: WorldQuery> WorldQuery for Option<T> {
    type Fetch = OptionFetch<T::Fetch>;
    type ReadOnlyFetch = OptionFetch<T::ReadOnlyFetch>;
}

pub struct OptionFetch<T>(PhantomData<fn() -> T>);

unsafe impl<'a, T: Fetch<'a>> Fetch<'a> for OptionFetch<T> {
    type Item = Option<T::Item>;

    fn access(access: &mut SystemAccess) {
        T::access(access);
    }

    fn entities(world: &'a World) -> Option<&'a EntitySet> {
        Some(world.entities().entities())
    }

    fn borrow(world: &World) -> bool {
        T::borrow(world)
    }

    unsafe fn get(
        world: &'a World,
        entity: &Entity,
        change_ticks: &ChangeTicks,
    ) -> Option<Self::Item> {
        unsafe { Some(T::get(world, entity, change_ticks)) }
    }

    fn release(world: &World) {
        T::release(world);
    }

    fn register_types(type_registry: &mut TypeRegistry) {
        T::register_types(type_registry);
    }
}

unsafe impl<'a, T: ReadOnlyFetch<'a>> ReadOnlyFetch<'a> for OptionFetch<T> {}

macro_rules! tuple_world_query {
    () => {};
    ($first:ident $(,$name:ident)* $(,)?) => {
        tuple_world_query!(@ $first, $($name),*);
        tuple_world_query!($($name),*);
    };
    (@ $($name:ident),* $(,)?) => {
        #[allow(non_snake_case)]
        unsafe impl<'a, $($name: Fetch<'a>),*> Fetch<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            fn access(access: &mut SystemAccess) {
                $($name::access(access);)*

            }

            fn entities(world: &'a World) -> Option<&'a EntitySet> {
                [$($name::entities(world)?),*].into_iter().min_by(|a, b| a.len().cmp(&b.len()))
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

            fn register_types(type_registry: &mut TypeRegistry) {
                $($name::register_types(type_registry);)*
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
