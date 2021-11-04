use std::slice::Iter as SliceIter;
use std::{any::TypeId, marker::PhantomData};

use crate::{Access, AnyComponent, Entity, SystemAccess, World};

pub trait Query {
    #[doc(hidden)]
    type Fetch: for<'a> Fetch<'a>;
}

pub type QueryItem<'a, Q> = <<Q as Query>::Fetch as Fetch<'a>>::Item;

pub unsafe trait Fetch<'a>: Sized {
    type Item;

    fn entities(world: &World) -> &[Entity];

    fn access(access: &mut SystemAccess);

    fn borrow(world: &World) -> bool;

    unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item>;

    fn release(world: &World);
}

impl<'a, T: AnyComponent> Query for &'a T {
    type Fetch = FetchRead<T>;
}

pub struct FetchRead<T>(PhantomData<fn() -> T>);

unsafe impl<'a, T: AnyComponent> Fetch<'a> for FetchRead<T> {
    type Item = &'a T;

    #[inline]
    fn entities(world: &World) -> &[Entity] {
        if let Some(storage) = world.components.get(&TypeId::of::<T>()) {
            storage.entities()
        } else {
            &[]
        }
    }

    #[inline]
    fn access(access: &mut SystemAccess) {
        access.borrow_component::<T>(Access::Read);
    }

    #[inline]
    fn borrow(world: &World) -> bool {
        world.borrow::<T>()
    }

    #[inline]
    unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
        let storage = world.components.get(&TypeId::of::<T>())?;

        unsafe { storage.get_unchecked(&entity) }
    }

    #[inline]
    fn release(world: &World) {
        world.release::<T>();
    }
}

impl<'a, T: AnyComponent> Query for &'a mut T {
    type Fetch = FetchWrite<T>;
}

pub struct FetchWrite<T>(PhantomData<fn() -> T>);

unsafe impl<'a, T: AnyComponent> Fetch<'a> for FetchWrite<T> {
    type Item = &'a mut T;

    #[inline]
    fn entities(world: &World) -> &[Entity] {
        if let Some(storage) = world.components.get(&TypeId::of::<T>()) {
            storage.entities()
        } else {
            &[]
        }
    }

    #[inline]
    fn access(access: &mut SystemAccess) {
        access.borrow_component::<T>(Access::Write);
    }

    #[inline]
    fn borrow(world: &World) -> bool {
        world.borrow_mut::<T>()
    }

    #[inline]
    unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
        let storage = world.components.get(&TypeId::of::<T>())?;

        unsafe { storage.get_unchecked_mut(&entity) }
    }

    #[inline]
    fn release(world: &World) {
        world.release_mut::<T>();
    }
}

impl Query for Entity {
    type Fetch = EntityFetch;
}

pub struct EntityFetch;

unsafe impl<'a> Fetch<'a> for EntityFetch {
    type Item = Entity;

    #[inline]
    fn entities(world: &World) -> &[Entity] {
        &world.entities
    }

    #[inline]
    fn access(_access: &mut SystemAccess) {}

    #[inline]
    fn borrow(_world: &World) -> bool {
        true
    }

    #[inline]
    unsafe fn get(_world: &'a World, entity: Entity) -> Option<Self::Item> {
        Some(entity)
    }

    #[inline]
    fn release(_world: &World) {}
}

impl<Q: Query, T: AnyComponent> Query for Without<Q, T> {
    type Fetch = Without<Q, T>;
}

pub struct Without<Q, T>(PhantomData<fn() -> (Q, T)>);

unsafe impl<'a, Q: Query, T: AnyComponent> Fetch<'a> for Without<Q, T> {
    type Item = QueryItem<'a, Q>;

    #[inline]
    fn entities(world: &World) -> &[Entity] {
        Q::Fetch::entities(world)
    }

    #[inline]
    fn access(access: &mut SystemAccess) {
        Q::Fetch::access(access);
    }

    #[inline]
    fn borrow(world: &World) -> bool {
        Q::Fetch::borrow(world)
    }

    #[inline]
    unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
        if world.has_component::<T>(&entity) {
            None
        } else {
            unsafe { Q::Fetch::get(world, entity) }
        }
    }

    #[inline]
    fn release(world: &World) {
        Q::Fetch::release(world)
    }
}

impl<Q: Query, T: AnyComponent> Query for With<Q, T> {
    type Fetch = With<Q, T>;
}

pub struct With<Q, T>(PhantomData<fn() -> (Q, T)>);

unsafe impl<'a, Q: Query, T: AnyComponent> Fetch<'a> for With<Q, T> {
    type Item = QueryItem<'a, Q>;

    #[inline]
    fn entities(world: &World) -> &[Entity] {
        Q::Fetch::entities(world)
    }

    #[inline]
    fn access(access: &mut SystemAccess) {
        Q::Fetch::access(access);
    }

    #[inline]
    fn borrow(world: &World) -> bool {
        Q::Fetch::borrow(world)
    }

    #[inline]
    unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
        if !world.has_component::<T>(&entity) {
            None
        } else {
            unsafe { Q::Fetch::get(world, entity) }
        }
    }

    #[inline]
    fn release(world: &World) {
        Q::Fetch::release(world)
    }
}

impl Query for () {
    type Fetch = ();
}

unsafe impl<'a> Fetch<'a> for () {
    type Item = ();

    #[inline]
    fn entities(world: &World) -> &[Entity] {
        &world.entities
    }

    #[inline]
    fn access(_access: &mut SystemAccess) {}

    #[inline]
    fn borrow(_world: &World) -> bool {
        true
    }

    #[inline]
    unsafe fn get(_world: &'a World, _entity: Entity) -> Option<Self::Item> {
        Some(())
    }

    #[inline]
    fn release(_world: &World) {}
}

pub struct QueryMut<'a, Q: Query> {
    entities: SliceIter<'a, Entity>,
    world: &'a World,
    fetch: PhantomData<Q::Fetch>,
}

impl<'a, Q: Query> QueryMut<'a, Q> {
    #[inline]
    pub fn new(world: &'a World) -> Option<Self> {
        if Q::Fetch::borrow(world) {
            Some(Self {
                entities: Q::Fetch::entities(world).into_iter(),
                world,
                fetch: PhantomData,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn get(&mut self, entity: Entity) -> Option<QueryItem<'_, Q>> {
        unsafe { Q::Fetch::get(self.world, entity) }
    }

    #[inline]
    pub fn without<T: AnyComponent>(mut self) -> QueryMut<'a, Without<Q, T>> {
        let query = QueryMut {
            entities: std::mem::replace(&mut self.entities, (&[]).into_iter()),
            world: self.world,
            fetch: PhantomData,
        };

        std::mem::forget(self);

        query
    }

    #[inline]
    pub fn with<T: AnyComponent>(mut self) -> QueryMut<'a, With<Q, T>> {
        let query = QueryMut {
            entities: std::mem::replace(&mut self.entities, (&[]).into_iter()),
            world: self.world,
            fetch: PhantomData,
        };

        std::mem::forget(self);

        query
    }
}

impl<'a, Q: Query> Iterator for QueryMut<'a, Q> {
    type Item = <Q::Fetch as Fetch<'a>>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.next() {
            let item = unsafe { Q::Fetch::get(self.world, *entity) };

            if item.is_some() {
                return item;
            }
        }

        None
    }
}

impl<'a, Q: Query> Drop for QueryMut<'a, Q> {
    #[inline]
    fn drop(&mut self) {
        Q::Fetch::release(self.world);
    }
}

macro_rules! tuple_impl {
	($($name:ident),*) => {
		unsafe impl<'a, $($name: Fetch<'a>),*> Fetch<'a> for ($($name,)*) {
			type Item = ($($name::Item,)*);

            #[inline]
            #[allow(unreachable_code)]
            fn entities(world: &World) -> &[Entity] {
                [$($name::entities(world)),*].into_iter().min_by(|a, b| a.len().cmp(&b.len())).unwrap()
            }

            #[inline]
			fn access(access: &mut SystemAccess) {
				$($name::access(access);)*
			}

            #[inline]
            #[allow(non_snake_case)]
			fn borrow(world: &World) -> bool {
				$(let $name = $name::borrow(world);)*

                let res = $($name)&&*;

                if !res {
                    $(
                        if $name {
                            $name::release(world);
                        }
                    )*
                }

                res
			}

            #[inline]
			unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
				Some(unsafe { ($($name::get(world, entity)?,)*) })
			}

            #[inline]
			fn release(world: &World) {
				$(
					$name::release(world);
				)*
			}
		}

		impl<'a, $($name: Query),*> Query for ($($name,)*) {
			type Fetch = ($($name::Fetch,)*);
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
