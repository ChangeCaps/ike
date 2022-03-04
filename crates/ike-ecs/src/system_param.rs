use std::marker::PhantomData;

use crate::{
    Access, ChangeTick, CommandQueue, Commands, Fetch, Query, QueryFilter, Res, ResMut, Resource,
    SystemAccess, World, WorldQuery,
};

pub trait SystemParam: Sized {
    type Fetch: for<'w, 's> SystemParamFetch<'w, 's>;
}

pub trait SystemParamFetch<'w, 's>: Sized + Send + Sync {
    type Item;

    fn init(world: &mut World) -> Self;

    fn access(access: &mut SystemAccess);

    fn get(&'s mut self, world: &'w World, last_change_tick: ChangeTick) -> Self::Item;

    fn apply(self, _world: &mut World) {}
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
}
