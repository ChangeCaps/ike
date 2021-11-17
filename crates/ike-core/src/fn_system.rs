use std::{any::type_name, marker::PhantomData};

use crate::{
    Access, CommandQueue, Commands, ExclusiveSystem, Fetch, Query, QueryFilter, ReadGuard,
    Resource, System, SystemAccess, World, WorldQuery, WorldRef, WriteGuard,
};

pub type Res<'a, T> = ReadGuard<'a, T>;
pub type ResMut<'a, T> = WriteGuard<'a, T>;

pub trait SystemParam: Sized {
    type Fetch: for<'w, 's> SystemParamFetch<'w, 's>;
}

pub trait SystemParamFetch<'w, 's>: Sized + Send + Sync {
    type Item;

    fn init(world: &mut World) -> Self;

    fn access(access: &mut SystemAccess);

    fn get(&'s mut self, world: &'w World, last_change_tick: u64) -> Self::Item;

    fn apply(self, world: &mut World);
}

impl<'a, Q: WorldQuery, F: QueryFilter> SystemParam for Query<'a, Q, F> {
    type Fetch = QueryFetch<Q, F>;
}

pub struct QueryFetch<Q, F>(PhantomData<fn() -> (Q, F)>);

impl<'w, 's, Q: WorldQuery, F: QueryFilter> SystemParamFetch<'w, 's> for QueryFetch<Q, F> {
    type Item = Query<'w, Q, F>;

    #[inline]
    fn init(_world: &mut World) -> Self {
        Self(PhantomData)
    }

    #[inline]
    fn access(access: &mut SystemAccess) {
        Q::Fetch::access(access);
    }

    #[inline]
    fn get(&'s mut self, world: &'w World, last_change_tick: u64) -> Self::Item {
        Query::new(world, last_change_tick).unwrap()
    }

    #[inline]
    fn apply(self, _world: &mut World) {}
}

impl<'a, T: Resource> SystemParam for Res<'a, T> {
    type Fetch = ResFetch<T>;
}

pub struct ResFetch<T>(PhantomData<fn() -> T>);

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResFetch<T> {
    type Item = Res<'w, T>;

    #[inline]
    fn init(_world: &mut World) -> Self {
        Self(PhantomData)
    }

    #[inline]
    fn access(access: &mut SystemAccess) {
        access.borrow_resource::<T>(Access::Read);
    }

    #[inline]
    fn get(&'s mut self, world: &'w World, _last_change_tick: u64) -> Self::Item {
        world.read_resource().unwrap()
    }

    #[inline]
    fn apply(self, _world: &mut World) {}
}

impl<'a, T: Resource> SystemParam for ResMut<'a, T> {
    type Fetch = ResMutFetch<T>;
}

pub struct ResMutFetch<T>(PhantomData<fn() -> T>);

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResMutFetch<T> {
    type Item = ResMut<'w, T>;

    #[inline]
    fn init(_world: &mut World) -> Self {
        Self(PhantomData)
    }

    #[inline]
    fn access(access: &mut SystemAccess) {
        access.borrow_resource::<T>(Access::Write);
    }

    #[inline]
    fn get(&'s mut self, world: &'w World, _last_change_tick: u64) -> Self::Item {
        world
            .write_resource()
            .expect(&format!("failed to get resource: '{}'", type_name::<T>()))
    }

    #[inline]
    fn apply(self, _world: &mut World) {}
}

impl<'w, 's> SystemParam for Commands<'w, 's> {
    type Fetch = CommandQueue;
}

impl<'w, 's> SystemParamFetch<'w, 's> for CommandQueue {
    type Item = Commands<'w, 's>;

    #[inline]
    fn init(_world: &mut World) -> Self {
        Self::default()
    }

    #[inline]
    fn access(_access: &mut SystemAccess) {}

    #[inline]
    fn get(&'s mut self, world: &'w World, _last_change_tick: u64) -> Self::Item {
        Commands::new(world.entities(), self)
    }

    #[inline]
    fn apply(self, world: &mut World) {
        self.apply(world);
    }
}

impl<'w, 's> SystemParam for WorldRef<'w, 's> {
    type Fetch = WorldFetch;
}

pub struct WorldFetch(CommandQueue);

impl<'w, 's> SystemParamFetch<'w, 's> for WorldFetch {
    type Item = WorldRef<'w, 's>;

    #[inline]
    fn init(_world: &mut World) -> Self {
        Self(CommandQueue::default())
    }

    #[inline]
    fn access(access: &mut SystemAccess) {
        access.borrow_world();
    }

    #[inline]
    fn get(&'s mut self, world: &'w World, last_change_tick: u64) -> Self::Item {
        WorldRef::new(
            world,
            Commands::new(world.entities(), &mut self.0),
            last_change_tick,
        )
    }

    #[inline]
    fn apply(self, world: &mut World) {
        self.0.apply(world);
    }
}

pub trait SystemParamFunc<Params: SystemParam>: Send + Sync + 'static {
    fn access() -> SystemAccess;

    fn run(&mut self, params: &mut Params::Fetch, world: &World, last_change_tick: u64);
}

macro_rules! impl_fn {
	($($name:ident),*) => {
        impl<$($name: SystemParam,)*> SystemParam for ($($name,)*) {
            type Fetch = ($($name::Fetch,)*);
        }

        #[allow(unused)]
        impl<'w, 's, $($name: SystemParamFetch<'w, 's>,)*> SystemParamFetch<'w, 's> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            #[inline]
            fn init(world: &mut World) -> Self {
                ($($name::init(world),)*)
            }

            #[inline]
            fn access(access: &mut SystemAccess) {
                $($name::access(access);)*
            }

            #[inline]
            fn get(&'s mut self, world: &'w World, last_change_tick: u64) -> Self::Item {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                ($($name.get(world, last_change_tick),)*)
            }

            #[inline]
            fn apply(self, world: &mut World) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $($name.apply(world);)*
            }
        }

		#[allow(unused)]
		impl<$($name: SystemParam + 'static,)* Func> SystemParamFunc<($($name,)*)> for Func
		where
            ($($name,)*): SystemParam<Fetch = ($($name::Fetch,)*)>,
			Func: Send + Sync + 'static,
			for<'a> &'a mut Func: FnMut($($name),*) +
			FnMut($(<<$name as SystemParam>::Fetch as SystemParamFetch>::Item),*),
		{
			#[inline]
			fn access() -> SystemAccess {
				let mut access = SystemAccess::default();

				$($name::Fetch::access(&mut access);)*

				access
			}

			#[inline]
            #[allow(non_snake_case)]
			fn run(&mut self, ($($name,)*): &mut <($($name,)*) as SystemParam>::Fetch, world: &World, last_change_tick: u64) {
				#[allow(non_snake_case)]
				fn call_inner<$($name),*>(mut f: impl FnMut($($name),*), $($name: $name),*) {
					f($($name),*);
				}

				call_inner(self, $(<$name::Fetch as SystemParamFetch>::get($name, world, last_change_tick)),*);
			}
		}
	};
}

pub trait FnSystem<Params> {
    type System: System;

    fn system(self) -> Self::System;
}

impl<F, Params> FnSystem<Params> for F
where
    F: SystemParamFunc<Params>,
    Params: SystemParam + 'static,
{
    type System = FuncSystem<F, Params>;

    #[inline]
    fn system(self) -> Self::System {
        FuncSystem {
            func: self,
            name: String::from(type_name::<F>()),
            state: None,
            last_change_tick: 0,
            marker: PhantomData,
        }
    }
}

pub struct FuncSystem<F, Params: SystemParam> {
    func: F,
    name: String,
    state: Option<Params::Fetch>,
    last_change_tick: u64,
    marker: PhantomData<fn() -> Params>,
}

impl<F, Params: 'static> System for FuncSystem<F, Params>
where
    F: SystemParamFunc<Params>,
    Params: SystemParam,
{
    #[inline]
    fn access(&self) -> SystemAccess {
        F::access()
    }

    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    fn run(&mut self, world: &World) {
        let last_change_tick = world.increment_change_tick();
        self.func.run(
            self.state
                .as_mut()
                .expect("init not called before system run"),
            world,
            self.last_change_tick,
        );
        self.last_change_tick = last_change_tick;
    }

    #[inline]
    fn init(&mut self, world: &mut World) {
        self.state = Some(Params::Fetch::init(world));
    }

    #[inline]
    fn apply(&mut self, world: &mut World) {
        self.state.take().unwrap().apply(world);
    }
}

pub trait ExclusiveFnSystem {
    type System: ExclusiveSystem;

    fn system(self) -> Self::System;
}

pub struct ExclusiveFuncSystem<F> {
    func: F,
    last_change_tick: u64,
}

impl<F: FnMut(&mut World) + Send + Sync + 'static> ExclusiveSystem for ExclusiveFuncSystem<F> {
    #[inline]
    fn run(&mut self, world: &mut World) {
        let last_change_tick = world.increment_change_tick();
        world.set_last_change_tick(self.last_change_tick);
        (self.func)(world);
        self.last_change_tick = last_change_tick;
    }
}

macro_rules! tuples {
	($macro:ident, $name:ident, $($names:ident),*) => {
		$macro!($name, $($names),*);
		tuples!($macro, $($names),*);
	};
	($macro:ident, $name:ident) => {
		$macro!($name);
		$macro!();
	};
}

tuples!(impl_fn, A, B, C, D, E, F, G, H, I, J, K);

#[cfg(test)]
mod tests {
    use crate::Entity;

    use super::*;

    #[test]
    fn system() {
        fn foo(query: Query<Entity>) {
            for _ in query {}
        }

        let _x = foo.system();
    }
}
