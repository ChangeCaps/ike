use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    Access, ChangeTick, CommandQueue, Commands, Fetch, FromWorld, Query, QueryFilter, Res, ResMut,
    Resource, SystemAccess, TypeRegistry, World, WorldQuery,
};

pub use ike_macro::SystemParam;

pub trait SystemParam: Sized {
    type Fetch: for<'w, 's> SystemParamFetch<'w, 's>;
}

pub trait SystemParamFetch<'w, 's>: Sized + Send + Sync {
    type Item;

    fn init(world: &mut World) -> Self;

    fn access(access: &mut SystemAccess);

    fn get(&'s mut self, world: &'w World, last_change_tick: ChangeTick) -> Self::Item;

    fn apply(self, _world: &mut World) {}

    fn register_types(_type_registry: &mut TypeRegistry);
}

pub struct Local<'a, T>(&'a mut T);

impl<'a, T> Deref for Local<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T> DerefMut for Local<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'a, T: Resource + FromWorld> SystemParam for Local<'a, T> {
    type Fetch = LocalState<T>;
}

pub struct LocalState<T>(T);

impl<'w, 's, T: Resource + FromWorld> SystemParamFetch<'w, 's> for LocalState<T> {
    type Item = Local<'s, T>;

    fn init(world: &mut World) -> Self {
        Self(T::from_world(world))
    }

    fn access(_: &mut SystemAccess) {}

    fn get(&'s mut self, _: &'w World, _: ChangeTick) -> Self::Item {
        Local(&mut self.0)
    }

    fn register_types(_type_registry: &mut TypeRegistry) {}
}

impl<'w, 's> SystemParam for Commands<'w, 's> {
    type Fetch = CommandsFetch;
}

pub struct CommandsFetch(CommandQueue);

impl<'w, 's> SystemParamFetch<'w, 's> for CommandsFetch {
    type Item = Commands<'w, 's>;

    fn init(_world: &mut World) -> Self {
        Self(CommandQueue::new())
    }

    fn access(_access: &mut SystemAccess) {}

    fn get(&'s mut self, world: &'w World, _last_change_tick: ChangeTick) -> Self::Item {
        Commands::new(world, &self.0)
    }

    fn apply(self, world: &mut World) {
        self.0.apply(world);
    }

    fn register_types(_type_registry: &mut TypeRegistry) {}
}

impl<'w, Q: WorldQuery, F: QueryFilter> SystemParam for Query<'w, Q, F> {
    type Fetch = QueryFetch<Q, F>;
}

pub struct QueryFetch<Q, F>(PhantomData<fn() -> (Q, F)>);

impl<'w, 's, Q: WorldQuery, F: QueryFilter> SystemParamFetch<'w, 's> for QueryFetch<Q, F> {
    type Item = Query<'w, Q, F>;

    fn init(_world: &mut World) -> Self {
        Self(PhantomData)
    }

    fn access(access: &mut SystemAccess) {
        Q::Fetch::access(access)
    }

    fn get(&'s mut self, world: &'w World, last_change_tick: ChangeTick) -> Self::Item {
        Query::new(world, last_change_tick)
            .expect("failed to borrow components for query, internal error")
    }

    fn register_types(type_registry: &mut TypeRegistry) {
        Q::Fetch::register_types(type_registry);
        Q::ReadOnlyFetch::register_types(type_registry);
    }
}

impl<'w, T: Resource> SystemParam for Res<'w, T> {
    type Fetch = ResFetch<T>;
}

pub struct ResFetch<T>(PhantomData<fn() -> T>);

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResFetch<T> {
    type Item = Res<'w, T>;

    fn init(_world: &mut World) -> Self {
        Self(PhantomData)
    }

    fn access(access: &mut SystemAccess) {
        access.borrow_resource::<T>(Access::Read);
    }

    fn get(&'s mut self, world: &'w World, _last_change_tick: ChangeTick) -> Self::Item {
        world.resource()
    }

    fn register_types(_type_registry: &mut TypeRegistry) {}
}

impl<'w, T: Resource> SystemParam for ResMut<'w, T> {
    type Fetch = ResMutFetch<T>;
}

pub struct ResMutFetch<T>(PhantomData<fn() -> T>);

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResMutFetch<T> {
    type Item = ResMut<'w, T>;

    fn init(_world: &mut World) -> Self {
        Self(PhantomData)
    }

    fn access(access: &mut SystemAccess) {
        access.borrow_resource::<T>(Access::Write);
    }

    fn get(&'s mut self, world: &'w World, _last_change_tick: ChangeTick) -> Self::Item {
        world.resource_mut()
    }

    fn register_types(_type_registry: &mut TypeRegistry) {}
}

macro_rules! impl_system_param {
    () => {
        impl_system_param!(@);
    };
    ($first:ident $(,$name:ident)*) => {
        impl_system_param!($($name),*);
        impl_system_param!(@ $first $(,$name)*);
    };
    (@ $($name:ident),*) => {
        #[allow(non_snake_case, unused)]
        impl<'w, 's, $($name: SystemParamFetch<'w, 's>),*> SystemParamFetch<'w, 's> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            fn init(world: &mut World) -> Self {
                ($($name::init(world),)*)
            }

            fn access(access: &mut SystemAccess) {
                $($name::access(access);)*
            }

            fn get(&'s mut self, world: &'w World, last_change_tick: ChangeTick) -> Self::Item {
                let ($($name,)*) = self;

                ($($name.get(world, last_change_tick),)*)
            }

            fn apply(self, world: &mut World) {
                let ($($name,)*) = self;

                $($name.apply(world);)*
            }

            fn register_types(type_registry: &mut TypeRegistry) {
                $($name::register_types(type_registry);)*
            }
        }

        impl<$($name: SystemParam),*> SystemParam for ($($name,)*) {
            type Fetch = ($($name::Fetch,)*);
        }
    };
}

impl_system_param!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
