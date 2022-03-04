use std::collections::BTreeSet;

use crate::{Entity, SystemAccess, World};

pub trait WorldQuery {
    type Fetch: for<'a> Fetch<'a>;
}

pub unsafe trait Fetch<'a>: Sized {
    type Item;

    fn entities(world: &World) -> &BTreeSet<Entity>;

    fn access(access: &mut SystemAccess);

    fn borrow(world: &World) -> bool;

    unsafe fn get(world: &'a World, entity: Entity) -> Option<Self::Item>;

    fn release(world: &World);
}

pub struct Query<'a> {
    entities: &'a BTreeSet<Entity>,
}
