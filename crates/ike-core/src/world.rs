use std::{any::TypeId, borrow::Cow, collections::HashMap, sync::atomic::{AtomicU64, Ordering}};

use crossbeam::queue::SegQueue;

use crate::{AnyComponent, BorrowLock, ComponentStorage, Entity, EntityRegistry, Node, OwnedComponent, Query, QueryFilter, QueryMut, ReadGuard, Resource, Resources, WriteGuard};

enum Command {
    Insert(Entity, OwnedComponent),
    InsertNode(Entity, String),
    InsertResource(TypeId, BorrowLock<dyn Resource>),
    RemoveResource(TypeId),
    InitResource(TypeId, BorrowLock<dyn Resource>),
}

pub struct World {
    pub(crate) components: HashMap<TypeId, ComponentStorage>,
    pub(crate) entities: Vec<Entity>,
    nodes: HashMap<Entity, String>,
    resources: Resources,
    commands: SegQueue<Command>,
    entity_registry: EntityRegistry,
    change_tick: AtomicU64,
    last_change_tick: u64,
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
            entities: Vec::new(),
            nodes: HashMap::new(),
            resources: Resources::new(),
            commands: SegQueue::new(),
            entity_registry: EntityRegistry::new(),
            change_tick: AtomicU64::new(1),
            last_change_tick: 0,
        }
    }

    #[inline]
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert(resource);
    }

    #[inline]
    pub fn init_resource<T: Resource + Default>(&mut self) {
        if !self.resources.contains::<T>() {
            self.resources.insert(T::default());
        }
    }

    #[inline]
    pub fn queue_insert_resource<T: Resource>(&self, resource: T) {
        self.commands.push(Command::InsertResource(
            TypeId::of::<T>(),
            BorrowLock::from_box(Box::new(resource)),
        ));
    }

    #[inline]
    pub fn queue_init_resource<T: Resource + Default>(&self) {
        if !self.resources.contains::<T>() {
            self.commands.push(Command::InitResource(
                TypeId::of::<T>(),
                BorrowLock::from_box(Box::new(T::default())),
            ));
        }
    }

    #[inline]
    pub fn queue_remove_resource<T: Resource>(&self) {
        self.commands
            .push(Command::RemoveResource(TypeId::of::<T>()));
    }

    #[inline]
    pub fn has_resource<T: Resource>(&self) -> bool {
        self.resources.contains::<T>()
    }

    #[inline]
    pub fn read_resource<T: Resource>(&self) -> Option<ReadGuard<T>> {
        self.resources.read()
    }

    #[inline]
    pub fn write_resource<T: Resource>(&self) -> Option<WriteGuard<T>> {
        self.resources.write()
    }

    #[inline]
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources.remove()
    }

    #[inline]
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    #[inline]
    pub fn create_entity(&self) -> Entity {
        self.entity_registry.next()
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
    pub fn query<Q: Query, F: QueryFilter>(&self) -> Option<QueryMut<'_, Q, F>> {
        QueryMut::new(self)
    }

    #[inline]
    pub fn has<T: AnyComponent>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn insert<T: AnyComponent>(&mut self, entity: Entity, component: T) {
        let change_tick = self.change_tick();

        let storage = self
            .components
            .entry(TypeId::of::<T>())
            .or_insert_with(ComponentStorage::new::<T>);

        // SAFETY: type in the storage matches T, since we got it with the TypeId.
        unsafe { storage.insert_unchecked(entity, component, change_tick) };
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
    pub fn contains_component<T: AnyComponent>(&self, entity: &Entity) -> bool {
        if let Some(storage) = self.components.get(&TypeId::of::<T>()) {
            storage.contains(entity)
        } else {
            false
        }
    }

    #[inline]
    pub fn get_component<T: AnyComponent>(&self, entity: &Entity) -> Option<ReadGuard<T>> {
        let storage = self.components.get(&TypeId::of::<T>())?;

        unsafe { storage.get_borrowed(entity) }
    }

    #[inline]
    pub fn get_component_mut<T: AnyComponent>(&self, entity: &Entity) -> Option<WriteGuard<T>> {
        let storage = self.components.get(&TypeId::of::<T>())?;

        unsafe { storage.get_borrowed_mut(entity, self.change_tick()) }
    }

    #[inline]
    pub fn dump_borrows(&self) {
        for storage in self.components.values() {
            if storage.borrow_mut() {
                storage.release_mut();

                println!("{:?} free", storage.ty());
            } else if storage.borrow() {
                storage.release();

                println!("{:?} shared", storage.ty());
            } else {
                println!("{:?} unique", storage.ty());
            }
        }
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
                    self.entities.push(entity);
                }
                Command::InsertResource(type_id, resource) => unsafe {
                    self.resources.insert_raw(type_id, resource);
                },
                Command::RemoveResource(type_id) => {
                    self.resources.remove_raw(type_id);
                }
                Command::InitResource(type_id, resource) => {
                    if !self.resources.contains_raw(type_id) {
                        unsafe { self.resources.insert_raw(type_id, resource) };
                    }
                }
            }
        }
    }

    #[inline]
    pub fn clear_trackers(&mut self) {
        self.last_change_tick = self.increment_change_tick();
    }

    #[inline]
    pub fn change_tick(&self) -> u64 {
        self.change_tick.load(Ordering::Acquire)
    }

    #[inline]
    pub fn last_change_tick(&self) -> u64 {
        self.last_change_tick
    }

    #[inline]
    pub fn increment_change_tick(&self) -> u64 {
        self.change_tick.fetch_add(1, Ordering::SeqCst)
    }

    #[inline]
    pub(crate) fn borrow<T: AnyComponent>(&self) -> bool {
        if let Some(storage) = self.components.get(&TypeId::of::<T>()) {
            storage.borrow()
        } else {
            true
        }
    }

    #[inline]
    pub(crate) fn borrow_mut<T: AnyComponent>(&self) -> bool {
        if let Some(storage) = self.components.get(&TypeId::of::<T>()) {
            storage.borrow_mut()
        } else {
            true
        }
    }

    #[inline]
    pub(crate) fn release<T: AnyComponent>(&self) {
        if let Some(storage) = self.components.get(&TypeId::of::<T>()) {
            storage.release();
        }
    }

    #[inline]
    pub(crate) fn release_mut<T: AnyComponent>(&self) {
        if let Some(storage) = self.components.get(&TypeId::of::<T>()) {
            storage.release_mut();
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

        node.insert(123i32);

        let e = node.entity();

        drop(node);

        world.dequeue();

        let node = world.get_node(e).unwrap();

        assert_eq!(*node.get_component::<i32>().unwrap(), 123);
    }

    #[test]
    fn query() {
        let mut world = World::new();

        let mut node = world.spawn_node("foo");

        node.insert(123i32);
        node.insert(false);

        drop(node);

        world.dequeue();

        let mut query = world.query::<(&i32, &bool), ()>().unwrap();

        assert_eq!(query.next(), Some((&123, &false)));
        assert_eq!(query.next(), None);

        assert!(world.query::<&bool, ()>().is_none());
    }
}
