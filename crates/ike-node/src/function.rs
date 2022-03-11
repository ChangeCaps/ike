use std::marker::PhantomData;

use ike_ecs::{Commands, CompMut, Component, Entity, With, World};

use crate::Node;

pub trait NodeFn<T> {
    fn run(&mut self, world: &World, commands: &Commands);
}

impl<F, T> NodeFn<T> for F
where
    F: FnMut(CompMut<T>, Node),
    T: Component,
{
    fn run(&mut self, world: &World, commands: &Commands) {
        for entity in world.query_filter::<Entity, With<T>>().unwrap().iter() {
            let node = Node::new(entity, world, commands);
            let component = node.component_mut::<T>();
            self(component, node);
        }
    }
}

pub trait NodeEventFn<T, E> {
    fn run(&mut self, world: &World, commands: &Commands, event: &E);

    fn run_single(&mut self, entity: Entity, world: &World, commands: &Commands, event: &E);
}

impl<F, T, E> NodeEventFn<T, E> for F
where
    F: FnMut(CompMut<T>, Node, &E),
    T: Component,
{
    fn run(&mut self, world: &World, commands: &Commands, event: &E) {
        for entity in world.query_filter::<Entity, With<T>>().unwrap().iter() {
            let node = Node::new(entity, world, commands);
            let component = node.component_mut::<T>();
            self(component, node, event);
        }
    }

    fn run_single(&mut self, entity: Entity, world: &World, commands: &Commands, event: &E) {
        if world.entities().contains_component::<T>(&entity) {
            let node = Node::new(entity, world, commands);
            let component = node.component_mut::<T>();
            self(component, node, event);
        }
    }
}

pub struct NodeEvent<F, T, E>(pub(crate) F, pub(crate) PhantomData<fn() -> (T, E)>)
where
    F: NodeEventFn<T, E>;

pub trait DynamicNodeEventFn<E> {
    fn run(&mut self, world: &World, commands: &Commands, event: &E);

    fn run_single(&mut self, entity: Entity, world: &World, commands: &Commands, event: &E);
}

impl<F, T, E> DynamicNodeEventFn<E> for NodeEvent<F, T, E>
where
    F: NodeEventFn<T, E>,
{
    fn run(&mut self, world: &World, commands: &Commands, event: &E) {
        self.0.run(world, commands, event)
    }

    fn run_single(&mut self, entity: Entity, world: &World, commands: &Commands, event: &E) {
        self.0.run_single(entity, world, commands, event)
    }
}
