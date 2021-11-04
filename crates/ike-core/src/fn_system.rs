use std::marker::PhantomData;

use crate::{
    Access, Fetch, Query, QueryMut, ReadGuard, Resource, System, SystemAccess, World, WriteGuard,
};

pub type Res<'a, T> = ReadGuard<'a, T>;
pub type ResMut<'a, T> = WriteGuard<'a, T>;

pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a>;
}

pub trait SystemParamFetch<'a> {
    type Item;

    fn access(access: &mut SystemAccess);

    fn get(world: &'a World) -> Self::Item;
}

impl<'a, Q: Query> SystemParam for QueryMut<'a, Q> {
    type Fetch = QueryFetch<Q>;
}

pub struct QueryFetch<Q>(PhantomData<fn() -> Q>);

impl<'a, Q: Query> SystemParamFetch<'a> for QueryFetch<Q> {
    type Item = QueryMut<'a, Q>;

    #[inline]
    fn access(access: &mut SystemAccess) {
        Q::Fetch::access(access);
    }

    #[inline]
    fn get(world: &'a World) -> Self::Item {
        world.query().unwrap()
    }
}

impl<'a, T: Resource> SystemParam for Res<'a, T> {
    type Fetch = ResFetch<T>;
}

pub struct ResFetch<T>(PhantomData<fn() -> T>);

impl<'a, T: Resource> SystemParamFetch<'a> for ResFetch<T> {
    type Item = Res<'a, T>;

    #[inline]
    fn access(access: &mut SystemAccess) {
        access.borrow_resource::<T>(Access::Read);
    }

    #[inline]
    fn get(world: &'a World) -> Self::Item {
        world.read_resource().unwrap()
    }
}

impl<'a, T: Resource> SystemParam for ResMut<'a, T> {
    type Fetch = ResMutFetch<T>;
}

pub struct ResMutFetch<T>(PhantomData<fn() -> T>);

impl<'a, T: Resource> SystemParamFetch<'a> for ResMutFetch<T> {
    type Item = ResMut<'a, T>;

    #[inline]
    fn access(access: &mut SystemAccess) {
        access.borrow_resource::<T>(Access::Write);
    }

    #[inline]
    fn get(world: &'a World) -> Self::Item {
        world.write_resource().unwrap()
    }
}

impl<'a> SystemParam for &'a World {
    type Fetch = WorldFetch;
}

pub struct WorldFetch;

impl<'a> SystemParamFetch<'a> for WorldFetch {
    type Item = &'a World;

    #[inline]
    fn access(access: &mut SystemAccess) {
        access.borrow_world();
    }

    #[inline]
    fn get(world: &'a World) -> Self::Item {
        world
    }
}

pub trait SystemParamFunc<Params>: Send + Sync + 'static {
    fn access() -> SystemAccess;

    fn run(&mut self, world: &World);
}

macro_rules! impl_fn {
	($($name:ident),*) => {
		#[allow(unused)]
		impl<$($name: SystemParam + 'static,)* Func> SystemParamFunc<($($name,)*)> for Func
		where
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
			fn run(&mut self, world: &World) {
				#[allow(non_snake_case)]
				fn call_inner<$($name),*>(mut f: impl FnMut($($name),*), $($name: $name),*) {
					f($($name),*);
				}

				call_inner(self, $($name::Fetch::get(world)),*);
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
    Params: 'static,
{
    type System = FuncSystem<F, Params>;

    #[inline]
    fn system(self) -> Self::System {
        FuncSystem {
            func: self,
            marker: PhantomData,
        }
    }
}

pub struct FuncSystem<F, Params> {
    func: F,
    marker: PhantomData<fn() -> Params>,
}

impl<F, Params: 'static> System for FuncSystem<F, Params>
where
    F: SystemParamFunc<Params>,
{
    #[inline]
    fn access(&self) -> SystemAccess {
        F::access()
    }

    #[inline]
    fn run(&mut self, world: &World) {
        self.func.run(world);
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
    use super::*;

    #[test]
    fn system() {
        fn foo(query: QueryMut<()>) {
            for _ in query {}
        }

        let _x = foo.system();
    }
}
