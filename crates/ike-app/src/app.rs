use std::mem;

use ike_ecs::{
    update_parent_system, Events, ExclusiveSystem, FromResources, Resource, System, SystemFn, World,
};

use crate::{AppRunner, AppStages, Plugin, RunOnce};

pub mod startup_stage {
    pub const PRE_STARTUP: &str = "pre_startup";
    pub const STARTUP: &str = "startup";
    pub const POST_STARTUP: &str = "post_startup";
}

pub mod stage {
    pub const START: &str = "start";
    pub const PRE_UPDATE: &str = "pre_update";
    pub const UPDATE: &str = "update";
    pub const POST_UPDATE: &str = "post_update";
    pub const END: &str = "end";
}

pub struct App {
    pub world: World,
    pub startup_stages: AppStages,
    pub stages: AppStages,
    pub runner: Box<dyn AppRunner>,
}

impl App {
    pub fn empty() -> Self {
        Self {
            world: World::new(),
            startup_stages: AppStages::new(),
            stages: AppStages::new(),
            runner: Box::new(RunOnce),
        }
    }

    /// Creates a new [`App`] with default [`stage`]s and [`update_parent_system`] at
    /// [`stage::POST_UPDATE`] and [`startup_stage::POST_STARTUP`].
    pub fn new() -> Self {
        let mut app = Self::empty();

        app.add_default_stages();

        app.add_startup_system_to_stage(update_parent_system.system(), startup_stage::POST_STARTUP);
        app.add_system_to_stage(update_parent_system.system(), stage::POST_UPDATE);

        app
    }

    /// Adds all stages in [`stage`] to self in the following order.
    ///
    /// Startup Stages:
    /// - [`startup_stage::PRE_STARTUP`]
    /// - [`startup_stage::STARTUP`]
    /// - [`startup_stage::POST_STARTUP`]
    ///
    /// Stages:
    /// - [`stage::START`]
    /// - [`stage::PRE_UPDATE`]
    /// - [`stage::UPDATE`]
    /// - [`stage::POST_UPDATE`]
    /// - [`stage::END`]
    ///
    /// # Panics
    /// - Panics if any of the stages already exist.
    pub fn add_default_stages(&mut self) {
        self.startup_stages
            .push_stage(startup_stage::PRE_STARTUP)
            .unwrap();
        self.startup_stages
            .push_stage(startup_stage::STARTUP)
            .unwrap();
        self.startup_stages
            .push_stage(startup_stage::POST_STARTUP)
            .unwrap();

        self.stages.push_stage(stage::START).unwrap();
        self.stages.push_stage(stage::PRE_UPDATE).unwrap();
        self.stages.push_stage(stage::UPDATE).unwrap();
        self.stages.push_stage(stage::POST_UPDATE).unwrap();
        self.stages.push_stage(stage::END).unwrap();
    }

    pub fn update(&mut self) {
        self.stages.execute(&mut self.world);
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);

        self
    }

    pub fn add_event<T: Resource>(&mut self) -> &mut Self {
        self.world.init_resource::<Events<T>>();
        self.add_system_to_stage(Events::<T>::update_system.system(), stage::END);

        self
    }

    pub fn with_runner(&mut self, runner: impl AppRunner + 'static) -> &mut Self {
        self.runner = Box::new(runner);

        self
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.world.insert_resource(resource);

        self
    }

    pub fn init_resource<T: Resource + FromResources>(&mut self) -> &mut Self {
        self.world.init_resource::<T>();

        self
    }

    /// Adds a new stage before `before`.
    ///
    /// # Panics
    /// - Panics if `before` doesn't exist.
    /// - Panics if `name` already exists.
    pub fn add_stage_before(
        &mut self,
        name: impl Into<String>,
        before: impl AsRef<str>,
    ) -> &mut Self {
        self.stages.insert_stage_before(name, before).unwrap();

        self
    }
    /// Adds a new stage before `after`.
    ///
    /// # Panics
    /// - Panics if `after` doesn't exist.
    /// - Panics if `name` already exists.
    pub fn add_stage_after(
        &mut self,
        name: impl Into<String>,
        after: impl AsRef<str>,
    ) -> &mut Self {
        self.stages.insert_stage_after(name, after).unwrap();

        self
    }

    /// Adds a [`System`] to `stage`.
    ///
    /// # Panics
    /// - Panics if `stage` doesn't exist.
    pub fn add_system_to_stage(
        &mut self,
        system: impl System,
        stage: impl AsRef<str>,
    ) -> &mut Self {
        self.stages.add_system_to_stage(system, stage).unwrap();

        self
    }

    /// Adds a [`System`] to [`stage::UPDATE`].
    ///
    /// # Panics
    /// - Panics if [`stage::UPDATE`] doesn't exist.
    pub fn add_system(&mut self, system: impl System) -> &mut Self {
        self.add_system_to_stage(system, stage::UPDATE);

        self
    }

    #[track_caller]
    pub fn add_exclusive_system_to_stage(
        &mut self,
        system: impl ExclusiveSystem,
        stage: impl AsRef<str>,
    ) -> &mut Self {
        self.stages
            .add_exclusive_system_to_stage(system, stage)
            .unwrap();

        self
    }

    pub fn add_exclusive_system(&mut self, system: impl ExclusiveSystem) -> &mut Self {
        self.add_exclusive_system_to_stage(system, stage::UPDATE);

        self
    }

    pub fn add_startup_system_to_stage(
        &mut self,
        system: impl System,
        stage: impl AsRef<str>,
    ) -> &mut Self {
        self.startup_stages
            .add_system_to_stage(system, stage)
            .unwrap();

        self
    }

    /// Adds a startup [`System`].
    pub fn add_startup_system(&mut self, system: impl System) -> &mut Self {
        self.add_startup_system_to_stage(system, startup_stage::STARTUP);

        self
    }

    /// Executes startup [`System`]s and runs `self.runner` if present.
    pub fn run(&mut self) {
        self.startup_stages.execute(&mut self.world);

        let mut app = mem::replace(self, App::empty());
        let runner = mem::replace(&mut app.runner, Box::new(RunOnce));
        runner.run(app);
    }
}
