use std::{any::TypeId, collections::HashMap};

use crate::{Component, ExclusiveSystem, Node, Plugin, Resource, Schedule, System, World};

pub mod stage {
    pub const START: &str = "start";
    pub const PRE_UPDATE: &str = "pre_update";
    pub const UPDATE: &str = "update";
    pub const POST_UPDATE: &str = "post_update";
    pub const RENDER: &str = "render";
    pub const END: &str = "end";
}

pub trait AppRunner: 'static {
    fn run(&mut self, app: App);
}

#[derive(Clone, Debug, Default)]
pub struct Time {
    time: f32,
    delta_time: f32,
}

impl Time {
    #[inline]
    pub fn advance_frame(&mut self, delta_time: f32) {
        self.time += delta_time;
        self.delta_time = delta_time;
    }

    #[inline]
    pub fn time_since_startup(&self) -> f32 {
        self.time
    }

    #[inline]
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    #[inline]
    pub fn frames_per_second(&self) -> f32 {
        1.0 / self.delta_time
    }
}

#[derive(Default)]
pub struct AppBuilder {
    app: App,
    runner: Option<Box<dyn AppRunner>>,
}

impl AppBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn world(&self) -> &World {
        &self.app.world
    }

    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.app.world
    }

    #[inline]
    pub fn add_stage(&mut self, stage: &'static str) -> &mut Self {
        self.app.stages.push((stage, Schedule::default()));
        self
    }

    #[inline]
    pub fn add_stage_before(&mut self, stage: &'static str, before: &'static str) -> &mut Self {
        let idx = self
            .app
            .stages
            .iter()
            .position(|(name, _)| *name == before)
            .expect("stage not found");

        self.app.stages.insert(idx, (stage, Schedule::default()));

        self
    }

    #[inline]
    pub fn add_stage_after(&mut self, stage: &'static str, after: &'static str) -> &mut Self {
        let idx = self
            .app
            .stages
            .iter()
            .position(|(name, _)| *name == after)
            .expect("stage not found");

        self.app
            .stages
            .insert(idx + 1, (stage, Schedule::default()));

        self
    }

    #[inline]
    pub fn get_stage_mut(&mut self, stage: &'static str) -> Option<&mut Schedule> {
        let idx = self
            .app
            .stages
            .iter()
            .position(|(name, _)| *name == stage)?;

        Some(&mut self.app.stages[idx].1)
    }

    #[inline]
    pub fn add_system_to_stage<T: System>(&mut self, system: T, stage: &'static str) -> &mut Self {
        let stage = self.get_stage_mut(stage).expect("stage not found");

        stage.add_system(system);

        self
    }

    #[inline]
    pub fn add_exclusive_system_to_stage<T: ExclusiveSystem>(
        &mut self,
        system: T,
        stage: &'static str,
    ) -> &mut Self {
        let stage = self.get_stage_mut(stage).expect("stage not found");

        stage.add_exclusive_system(system);

        self
    }

    #[inline]
    pub fn add_system<T: System>(&mut self, system: T) -> &mut Self {
        self.add_system_to_stage(system, stage::UPDATE);
        self
    }

    #[inline]
    pub fn add_startup_system<T: System>(&mut self, system: T) -> &mut Self {
        self.app.startup.add_system(system);
        self
    }

    #[inline]
    pub fn add_exclusive_startup_system<T: ExclusiveSystem>(&mut self, system: T) -> &mut Self {
        self.app.startup.add_exclusive_system(system);
        self
    }

    #[inline]
    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.world_mut().insert_resource(resource);
        self
    }

    #[inline]
    pub fn init_resource<T: Resource + Default>(&mut self) -> &mut Self {
        self.world_mut().init_resource::<T>();
        self
    }

    #[inline]
    pub fn register_component<T: Component>(&mut self) -> &mut Self {
        fn update<T: Component>(node: &mut Node<'_>) {
            let mut component = node.world().get_component_mut::<T>(&node.entity()).unwrap();

            component.update(node, node.world());
        }

        self.app.components.insert(TypeId::of::<T>(), update::<T>);

        self
    }

    #[inline]
    pub fn set_runner<T: AppRunner>(&mut self, runner: T) -> &mut Self {
        self.runner = Some(Box::new(runner));
        self
    }

    #[inline]
    pub fn add_plugin<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        plugin.build(self);
        self
    }

    #[inline]
    pub fn run(&mut self) {
        if let Some(mut runner) = self.runner.take() {
            runner.run(std::mem::replace(&mut self.app, App::default()));
        }
    }
}

#[derive(Default)]
pub struct App {
    world: World,
    components: HashMap<TypeId, fn(&mut Node)>,
    startup: Schedule,
    stages: Vec<(&'static str, Schedule)>,
}

impl App {
    #[inline]
    pub fn new() -> AppBuilder {
        let mut builder = AppBuilder::new();

        builder.add_stage(stage::START);
        builder.add_stage(stage::PRE_UPDATE);
        builder.add_stage(stage::UPDATE);
        builder.add_stage(stage::POST_UPDATE);
        builder.add_stage(stage::RENDER);
        builder.add_stage(stage::END);

        builder
    }

    #[inline]
    pub fn world(&self) -> &World {
        &self.world
    }

    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    #[inline]
    pub fn execute_startup(&mut self) {
        self.startup.execute(&mut self.world);
    }

    #[inline]
    pub fn execute(&mut self) {
        for (_, stage) in &mut self.stages {
            stage.execute(&mut self.world);
        }
    }

    #[inline]
    pub fn update_components(&mut self) {
        for (id, update) in self.components.iter() {
            if let Some(storage) = self.world.components.get(id) {
                for entity in storage.entities() {
                    let mut node = self.world.get_node(*entity).unwrap();

                    update(&mut node);
                }
            }
        }

        self.world.dequeue();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn temp() {
        fn t<'a>(a: &*mut *const f32, x: &'a f32) -> &'a Repr {
            unsafe { **a = x as *const _ };

            unsafe { std::mem::transmute(&**a) }
        }

        #[repr(transparent)]
        struct Repr(*const f32);

        let f = 2.2f32;
        let a = Box::into_raw(Box::new(std::ptr::null()));

        let r = t(&a, &f);

        let x = unsafe { &*r.0 };
        println!("{}", x);

        unsafe { Box::from_raw(a) };
    }
}
