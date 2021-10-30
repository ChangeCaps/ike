use std::{
    any::TypeId,
    borrow::Cow,
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crossbeam::queue::SegQueue;

use crate::{AnyComponent, BorrowLock, ComponentStorage, Entity, Node, OwnedComponent, System};

enum Command {
    Insert(Entity, OwnedComponent),
    InsertNode(Entity, String),
}

pub struct World {
    pub(crate) components: HashMap<TypeId, ComponentStorage>,
    nodes: HashMap<Entity, String>,
    systems: HashMap<TypeId, BorrowLock<dyn System>>,
    commands: SegQueue<Command>,
    next_entity: AtomicU64,
}

impl Default for World {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    #[inline]
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            nodes: HashMap::new(),
            systems: HashMap::new(),
            commands: SegQueue::new(),
            next_entity: AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn create_entity(&self) -> Entity {
        let raw = self.next_entity.fetch_add(1, Ordering::Acquire);
        Entity::from_raw(raw)
    }

    #[inline]
    pub fn set_node_name(&mut self, entity: &Entity, name: impl Into<String>) {
        self.nodes
            .get_mut(entity)
            .map(|node_name| *node_name = name.into());
    }

    #[inline]
    pub fn queue_set_node_name(&self, entity: Entity, name: impl Into<String>) {
        self.commands.push(Command::InsertNode(entity, name.into()));
    }

    #[inline]
    pub fn spawn_node(&self, name: impl Into<String>) -> Node {
        let name = name.into();

        let entity = self.create_entity();

        self.queue_set_node_name(entity, name.clone());

        Node {
            name: Cow::Owned(name.into()),
            entity,
            owned: HashMap::new(),
            world: self,
        }
    }

    #[inline]
    pub fn get_node(&self, entity: Entity) -> Option<Node> {
        let name = self.nodes.get(&entity)?;

        Some(Node {
            name: Cow::Borrowed(name),
            entity,
            owned: HashMap::new(),
            world: self,
        })
    }

    #[inline]
    pub fn spawn(&mut self) {}

    #[inline]
    pub fn insert<T: AnyComponent>(&mut self, entity: Entity, component: T) {
        let storage = self
            .components
            .entry(TypeId::of::<T>())
            .or_insert_with(ComponentStorage::new::<T>);

        // SAFETY: type in the storage matches T, since we got it with the TypeId.
        unsafe { storage.insert_unchecked(entity, component) };
    }

    #[inline]
    pub fn queue_insert<T: AnyComponent>(&self, entity: Entity, component: T) {
        self.commands
            .push(Command::Insert(entity, OwnedComponent::new(component)));
    }

    #[inline]
    pub(crate) fn queue_insert_raw(&self, entity: Entity, component: OwnedComponent) {
        self.commands.push(Command::Insert(entity, component));
    }

    #[inline]
    pub fn nodes(&self) -> impl Iterator<Item = Node> + '_ {
        self.nodes.iter().map(|(entity, name)| Node {
            name: Cow::Borrowed(name),
            entity: *entity,
            owned: HashMap::new(),
            world: self,
        })
    }

    #[inline]
    pub fn dequeue(&mut self) {
        while let Some(command) = self.commands.pop() {
            match command {
                Command::Insert(entity, component) => component.insert(entity, self),
                Command::InsertNode(entity, name) => {
                    self.nodes.insert(entity, name);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities() {
        let mut world = World::new();

        let e = world.create_entity();

        world.insert(e, 64i32);
        world.insert(e, false)
    }

    #[test]
    fn node() {
        let mut world = World::new();

        let mut node = world.spawn_node("foo");

        node.queue_insert(123i32);

        let e = node.entity();

        drop(node);

        world.dequeue();

        let node = world.get_node(e).unwrap();

        assert_eq!(*node.get_component::<i32>().unwrap(), 123);
    }
}
