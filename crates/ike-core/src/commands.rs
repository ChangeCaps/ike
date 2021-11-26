use std::{any::TypeId, marker::PhantomData};

use crossbeam::queue::SegQueue;

use crate::{AnyComponent, Entities, Entity, Resource, World};

pub trait Command: Send + Sync + 'static {
    fn apply(self: Box<Self>, world: &mut World);
}

#[derive(Default)]
pub struct CommandQueue {
    commands: SegQueue<Box<dyn Command>>,
}

impl CommandQueue {
    #[inline]
    pub fn apply(self, world: &mut World) {
        while let Some(command) = self.commands.pop() {
            command.apply(world);
        }
    }
}

pub struct Commands<'w, 's> {
    entities: &'w Entities,
    command_queue: &'s mut CommandQueue,
}

impl<'w, 's> Commands<'w, 's> {
    #[inline]
    pub fn new(entities: &'w Entities, command_queue: &'s mut CommandQueue) -> Self {
        Self {
            entities,
            command_queue,
        }
    }

    #[inline]
    pub fn push<T: Command>(&self, command: T) {
        self.command_queue.commands.push(Box::new(command));
    }

    #[inline]
    pub fn spawn_node(&self, name: impl Into<String>) -> crate::SpawnNode<'_, '_> {
        let entity = self.entities.reserve_entity();
        self.push(SpawnNode(entity, name.into()));

        crate::SpawnNode::new(entity, self)
    }

    #[inline]
    pub fn set_node_name(&self, entity: &Entity, name: impl Into<String>) {
        self.push(SetNodeName(*entity, name.into()));
    }

    #[inline]
    pub fn despawn(&self, entity: &Entity) {
        self.push(Despawn(*entity));
    }

    #[inline]
    pub fn insert_component<T: AnyComponent>(&self, entity: &Entity, component: T) {
        self.push(Insert(*entity, component));
    }

    #[inline]
    pub fn remove_component<T: AnyComponent>(&self, entity: &Entity) {
        self.push(Remove::<T>(*entity, PhantomData));
    }

    #[inline]
    pub fn remove_component_raw(&self, entity: &Entity, type_id: &TypeId) {
        self.push(RemoveRaw(*entity, *type_id));
    }

    #[inline]
    pub fn insert_resource<T: Resource>(&self, resource: T) {
        self.push(InsertResource(resource));
    }

    #[inline]
    pub fn init_resource<T: Resource + Default>(&self) {
        self.push(InitResource::<T>(PhantomData));
    }

    #[inline]
    pub fn remove_resource<T: Resource>(&self) {
        self.push(RemoveResource::<T>(PhantomData));
    }
}

struct SpawnNode(Entity, String);

impl Command for SpawnNode {
    fn apply(self: Box<Self>, world: &mut World) {
        world.entities_mut().spawn(self.0);
        world.set_node_name(&self.0, self.1);
    }
}

struct SetNodeName(Entity, String);

impl Command for SetNodeName {
    fn apply(self: Box<Self>, world: &mut World) {
        world.set_node_name(&self.0, self.1);
    }
}

struct Despawn(Entity);

impl Command for Despawn {
    fn apply(self: Box<Self>, world: &mut World) {
        world.remove_node(&self.0);
    }
}

struct Insert<T>(Entity, T);

impl<T: AnyComponent> Command for Insert<T> {
    fn apply(self: Box<Self>, world: &mut World) {
        let change_tick = world.change_tick();
        world.entities_mut().insert(self.0, self.1, change_tick);
    }
}

struct Remove<T>(Entity, PhantomData<fn() -> T>);

impl<T: AnyComponent> Command for Remove<T> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.entities_mut().remove::<T>(&self.0);
    }
}

struct RemoveRaw(Entity, TypeId);

impl Command for RemoveRaw {
    fn apply(self: Box<Self>, world: &mut World) {
        if let Some(storage) = world.entities_mut().storage_raw_mut(&self.1) {
            unsafe { storage.remove_unchecked_raw(&self.0) };
        }
    }
}

struct InsertResource<T>(T);

impl<T: Resource> Command for InsertResource<T> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.insert_resource(self.0);
    }
}

struct InitResource<T>(PhantomData<fn() -> T>);

impl<T: Resource + Default> Command for InitResource<T> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.init_resource::<T>();
    }
}

struct RemoveResource<T>(PhantomData<fn() -> T>);

impl<T: Resource> Command for RemoveResource<T> {
    fn apply(self: Box<Self>, world: &mut World) {
        world.remove_resource::<T>();
    }
}
