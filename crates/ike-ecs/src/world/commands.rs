use std::marker::PhantomData;

use crossbeam::queue::SegQueue;

use crate::{Children, Component, Entity, EntityCommands, Resource, World};

pub trait Command: Send + Sync + 'static {
    fn run(self: Box<Self>, world: &mut World);
}

pub struct CommandQueue {
    commands: SegQueue<Box<dyn Command>>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: SegQueue::new(),
        }
    }

    pub fn push<T: Command>(&self, command: T) {
        self.commands.push(Box::new(command));
    }

    pub fn apply(self, world: &mut World) {
        while let Some(command) = self.commands.pop() {
            command.run(world);
        }
    }
}

pub struct Commands<'w, 's> {
    world: &'w World,
    queue: &'s CommandQueue,
}

impl<'w, 's> Commands<'w, 's> {
    pub const fn new(world: &'w World, queue: &'s CommandQueue) -> Self {
        Self { world, queue }
    }

    pub fn push<T: Command>(&self, command: T) {
        self.queue.push(command);
    }

    pub fn spawn(&self) -> EntityCommands {
        let entity = self.world.entities().reserve();

        self.push(Spawn(entity));

        self.entity(&entity)
    }

    pub fn entity(&self, entity: &Entity) -> EntityCommands {
        EntityCommands::new(self, *entity)
    }

    pub fn insert<T: Component>(&self, entity: &Entity, component: T) {
        self.push(Insert(*entity, component));
    }

    pub fn remove<T: Component>(&self, entity: &Entity) {
        self.push(Remove::<T>(*entity, PhantomData));
    }

    pub fn insert_resource<T: Resource>(&self, resource: T) {
        self.push(InsertResource(resource));
    }

    pub fn remove_resource<T: Resource>(&self) {
        self.push(RemoveResource::<T>(PhantomData));
    }

    pub fn despawn(&self, entity: &Entity) {
        self.push(Despawn(*entity));
    }

    pub fn despawn_recursive(&self, entity: &Entity) {
        self.push(DespawnRecursive(*entity));
    }
}

struct Spawn(Entity);

impl Command for Spawn {
    fn run(self: Box<Self>, world: &mut World) {
        world.entities_mut().spawn_reserved_entity(self.0);
    }
}

struct Insert<T>(Entity, T);

impl<T: Component> Command for Insert<T> {
    fn run(self: Box<Self>, world: &mut World) {
        let change_tick = world.change_tick();
        world.entities_mut().insert(&self.0, self.1, change_tick);
    }
}

struct Remove<T>(Entity, PhantomData<fn() -> T>);

impl<T: Component> Command for Remove<T> {
    fn run(self: Box<Self>, world: &mut World) {
        world.entities_mut().remove::<T>(&self.0);
    }
}

struct InsertResource<T>(T);

impl<T: Resource> Command for InsertResource<T> {
    fn run(self: Box<Self>, world: &mut World) {
        world.insert_resource(self.0);
    }
}

struct RemoveResource<T>(PhantomData<fn() -> T>);

impl<T: Resource> Command for RemoveResource<T> {
    fn run(self: Box<Self>, world: &mut World) {
        world.remove_resource::<T>();
    }
}

struct Despawn(Entity);

impl Command for Despawn {
    fn run(self: Box<Self>, world: &mut World) {
        world.entities_mut().despawn(&self.0);
    }
}

struct DespawnRecursive(Entity);

impl Command for DespawnRecursive {
    fn run(self: Box<Self>, world: &mut World) {
        despawn_recursive(world, self.0);
    }
}

fn despawn_recursive(world: &mut World, entity: Entity) {
    if let Some(mut children) = world.get_component_mut::<Children>(&entity) {
        for entity in std::mem::take(&mut children.children) {
            despawn_recursive(world, entity);
        }
    }

    world.despawn(&entity);
}
