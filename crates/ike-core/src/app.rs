use std::{any::TypeId, collections::HashMap};

use crate::{Component, ExclusiveSystem, Node, Plugin, Schedule, System, World};

pub trait AppRunner: 'static {
    fn run(&mut self, app: App);
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
    pub fn schedule(&self) -> &Schedule {
        &self.app.schedule
    }

    #[inline]
    pub fn schedule_mut(&mut self) -> &mut Schedule {
        &mut self.app.schedule
    }

    #[inline]
    pub fn add_system<T: System>(&mut self, system: T) -> &mut Self {
        self.app.schedule.add_system(system);
        self
    }

    #[inline]
    pub fn add_exclusive_system<T: ExclusiveSystem>(&mut self, system: T) -> &mut Self {
        self.app.schedule.add_exclusive_system(system);
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
    schedule: Schedule,
}

impl App {
    #[inline]
    pub fn new() -> AppBuilder {
        AppBuilder::new()
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
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    #[inline]
    pub fn schedule_mut(&mut self) -> &mut Schedule {
        &mut self.schedule
    }

    #[inline]
    pub fn execute_schedule(&mut self) {
        self.schedule.execute(&mut self.world);
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
