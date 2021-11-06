use std::ops::{Deref, DerefMut};
use std::slice::Iter as SliceIter;
use std::sync::atomic::AtomicU64;
use std::marker::PhantomData;

use crate::{Access, AnyComponent, Entity, QueryFilter, SystemAccess, World};

pub trait WorldQuery {
    #[doc(hidden)]
    type Fetch: for<'a> Fetch<'a>;
}

pub type QueryItem<'a, Q> = <<Q as WorldQuery>::Fetch as Fetch<'a>>::Item;

pub unsafe trait Fetch<'a>: Sized {
    type Item;

    fn entities(world: &World) -> &[Entity];

    fn access(access: &mut SystemAccess);

    fn borrow(world: &World) -> bool;

    unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item>;

    fn release(world: &World);
}

impl<'a, T: AnyComponent> WorldQuery for &'a T {
    type Fetch = FetchRead<T>;
}

pub struct FetchRead<T>(PhantomData<fn() -> T>);

unsafe impl<'a, T: AnyComponent> Fetch<'a> for FetchRead<T> {
    type Item = &'a T;

    #[inline]
    fn entities(world: &World) -> &[Entity] {
        if let Some(storage) = world.entities().storage::<T>() {
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
        world.entities().borrow::<T>()
    }

    #[inline]
    unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
        let storage = world.entities().storage::<T>()?;

        unsafe { storage.get_unchecked(&entity) }
    }

    #[inline]
    fn release(world: &World) {
        world.entities().release::<T>();
    }
}

pub struct Mut<'a, T> {
    inner: &'a mut T,
    changed: &'a AtomicU64,
    change_frame: u64,
}

impl<'a, T> Mut<'a, T> {
    #[inline]
    pub fn unmarked(&mut self) -> &mut T {
        self.inner
    }

    #[inline]
    pub fn mark_changed(&self) {
        self.changed
            .store(self.change_frame, std::sync::atomic::Ordering::Release);
    }
}

impl<'a, T: PartialEq> PartialEq for Mut<'a, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (&*self.inner).eq(other.inner)
    }
}

impl<'a, T: Eq> Eq for Mut<'a, T> {}

impl<'a, T> Deref for Mut<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for Mut<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mark_changed();

        self.inner
    }
}

impl<'a, T: AnyComponent> WorldQuery for &'a mut T {
    type Fetch = FetchWrite<T>;
}

pub struct FetchWrite<T>(PhantomData<fn() -> T>);

unsafe impl<'a, T: AnyComponent> Fetch<'a> for FetchWrite<T> {
    type Item = Mut<'a, T>;

    #[inline]
    fn entities(world: &World) -> &[Entity] {
        if let Some(storage) = world.entities().storage::<T>() {
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
        world.entities().borrow_mut::<T>()
    }

    #[inline]
    unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
        let storage = world.entities().storage::<T>()?;

        let component = unsafe { storage.get_unchecked_mut(&entity)? };

        Some(Mut {
            inner: component,
            changed: storage.get_change_marker(&entity)?,
            change_frame: world.change_tick(),
        })
    }

    #[inline]
    fn release(world: &World) {
        world.entities().release_mut::<T>();
    }
}

impl WorldQuery for Entity {
    type Fetch = EntityFetch;
}

pub struct EntityFetch;

unsafe impl<'a> Fetch<'a> for EntityFetch {
    type Item = Entity;

    #[inline]
    fn entities(world: &World) -> &[Entity] {
        world.entities().entities()
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

pub struct Query<'a, Q: WorldQuery, F: QueryFilter = ()> {
    entities: SliceIter<'a, Entity>,
    world: &'a World,
    last_change_tick: u64,
    marker: PhantomData<(Q::Fetch, F)>,
}

impl<'a, Q: WorldQuery, F: QueryFilter> Query<'a, Q, F> {
    #[inline]
    pub fn new(world: &'a World, last_change_tick: u64) -> Option<Self> {
        if Q::Fetch::borrow(world) {
            Some(Self {
                entities: Q::Fetch::entities(world).into_iter(),
                last_change_tick,
                world,
                marker: PhantomData,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn get(&mut self, entity: Entity) -> Option<QueryItem<'a, Q>> {
        if !F::filter(self.world, &entity, self.last_change_tick) {
            return None;
        }

        unsafe { Q::Fetch::get(self.world, entity) }
    }
}

impl<'a, Q: WorldQuery, F: QueryFilter> Iterator for Query<'a, Q, F> {
    type Item = <Q::Fetch as Fetch<'a>>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.next() {
            if let Some(item) = self.get(*entity) {
                return Some(item);
            }
        }

        None
    }
}

impl<'a, Q: WorldQuery, F: QueryFilter> Drop for Query<'a, Q, F> {
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

		impl<'a, $($name: WorldQuery),*> WorldQuery for ($($name,)*) {
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
