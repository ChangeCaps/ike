use std::marker::PhantomData;

use crossbeam::queue::SegQueue;

use crate::{Component, Entity, SpawnEntity, World};

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

    pub fn spawn(&self) -> SpawnEntity<'_, '_> {
        let entity = self.world.entities().reserve();

        self.push(Spawn(entity));

        SpawnEntity::new(self, entity)
    }

    pub fn insert<T: Component>(&self, entity: &Entity, component: T) {
        self.push(Insert(*entity, component));
    }

    pub fn remove<T: Component>(&self, entity: &Entity) {
        self.push(Remove::<T>(*entity, PhantomData));
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
