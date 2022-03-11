use std::{any::TypeId, collections::HashMap, marker::PhantomData};

use ike_ecs::{Commands, CompMut, Component, Entity, Resource, World};

use crate::{DynamicNodeEventFn, Node, NodeEvent, NodeEventFn, NodeFn};

#[derive(Default)]
pub struct NodeStages {
    stages: HashMap<NodeStageName, NodeStage>,
    event_stages: HashMap<NodeEventStageName, Box<dyn DynamicNodeEventStage>>,
}

impl NodeStages {
    pub fn get_stage(&self, name: &'static str) -> Option<&NodeStage> {
        self.stages.get(&NodeStageName(name))
    }

    pub fn get_stage_mut(&mut self, name: &'static str) -> Option<&mut NodeStage> {
        self.stages.get_mut(&NodeStageName(name))
    }

    pub fn get_event_stage<E: Resource>(
        &mut self,
        name: &'static str,
    ) -> Option<&NodeEventStage<E>> {
        let name = NodeEventStageName {
            event_type: TypeId::of::<E>(),
            name,
        };

        unsafe { Some(&*(self.event_stages.get_mut(&name)?.as_ref() as *const _ as *const _)) }
    }

    pub fn get_event_stage_mut<E: Resource>(
        &mut self,
        name: &'static str,
    ) -> Option<&mut NodeEventStage<E>> {
        let name = NodeEventStageName {
            event_type: TypeId::of::<E>(),
            name,
        };

        unsafe { Some(&mut *(self.event_stages.get_mut(&name)?.as_mut() as *mut _ as *mut _)) }
    }

    pub fn add_stage_fn<T: Component>(
        &mut self,
        name: &'static str,
        func: impl FnMut(CompMut<T>, Node) + Send + Sync + 'static,
    ) {
        let stage = self.stages.entry(NodeStageName(name)).or_default();
        stage.add_fn(func);
    }

    pub fn add_event_fn<T: Component, E: Resource>(
        &mut self,
        name: &'static str,
        func: impl FnMut(CompMut<T>, Node, &E) + Send + Sync + 'static,
    ) {
        let name = NodeEventStageName {
            event_type: TypeId::of::<E>(),
            name,
        };

        let stage = self
            .event_stages
            .entry(name)
            .or_insert_with(|| Box::new(NodeEventStage::<E>::new()));

        unsafe { (&mut *(stage.as_mut() as *mut _ as *mut NodeEventStage<E>)).add_fn(func) };
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeStageName(&'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeEventStageName {
    event_type: TypeId,
    name: &'static str,
}

#[derive(Default)]
pub struct NodeStage {
    functions: Vec<Box<dyn FnMut(&World, &Commands) + Send + Sync>>,
}

impl NodeStage {
    pub fn add_fn<T>(&mut self, mut func: impl NodeFn<T> + Send + Sync + 'static) {
        self.functions
            .push(Box::new(move |world, commands| func.run(world, commands)))
    }

    pub fn run(&mut self, world: &World, commands: &Commands) {
        for function in self.functions.iter_mut() {
            function(world, commands);
        }
    }
}

pub struct NodeEventStage<E> {
    functions: Vec<Box<dyn DynamicNodeEventFn<E> + Send + Sync + 'static>>,
}

impl<E: Resource> NodeEventStage<E> {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    pub fn add_fn<T: Component>(&mut self, func: impl NodeEventFn<T, E> + Send + Sync + 'static) {
        self.functions.push(Box::new(NodeEvent(func, PhantomData)));
    }

    pub fn run(&mut self, world: &World, commands: &Commands, event: &E) {
        for function in self.functions.iter_mut() {
            function.run(world, commands, event);
        }
    }

    pub fn run_single(&mut self, entity: Entity, world: &World, commands: &Commands, event: &E) {
        for function in self.functions.iter_mut() {
            function.run_single(entity, world, commands, event);
        }
    }
}

impl<E: Resource> DynamicNodeEventStage for NodeEventStage<E> {}

pub trait DynamicNodeEventStage: Send + Sync + 'static {}
