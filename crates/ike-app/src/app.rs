use std::mem;

use ike_ecs::{
    update_parent_system, Events, FromWorld, IntoSystemDescriptor, Resource, Schedule, StageLabel,
    World,
};

use crate::{AppRunner, Plugin, RunOnce};

#[derive(Clone, Copy, Debug, Hash, StageLabel)]
pub enum StartupStage {
    PreStartup,
    Startup,
    PostStartup,
}

#[derive(Clone, Copy, Debug, Hash, StageLabel)]
pub enum CoreStage {
    Start,
    PreUpdate,
    Update,
    PostUpdate,
    End,
}

pub struct App {
    pub world: World,
    pub startup: Schedule,
    pub schedule: Schedule,
    pub runner: Box<dyn AppRunner>,
}

impl App {
    pub fn empty() -> Self {
        Self {
            world: World::new(),
            startup: Schedule::new(),
            schedule: Schedule::new(),
            runner: Box::new(RunOnce),
        }
    }

    /// Creates a new [`App`] with default [`stage`]s and [`update_parent_system`] at
    /// [`stage::POST_UPDATE`] and [`startup_stage::POST_STARTUP`].
    pub fn new() -> Self {
        let mut app = Self::empty();

        app.add_default_stages();

        app.add_startup_system_to_stage(update_parent_system, StartupStage::PostStartup);
        app.add_system_to_stage(update_parent_system, CoreStage::PostUpdate);

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
        self.startup.push_stage(StartupStage::PreStartup).unwrap();
        self.startup.push_stage(StartupStage::Startup).unwrap();
        self.startup.push_stage(StartupStage::PostStartup).unwrap();

        self.schedule.push_stage(CoreStage::Start).unwrap();
        self.schedule.push_stage(CoreStage::PreUpdate).unwrap();
        self.schedule.push_stage(CoreStage::Update).unwrap();
        self.schedule.push_stage(CoreStage::PostUpdate).unwrap();
        self.schedule.push_stage(CoreStage::End).unwrap();
    }

    pub fn update(&mut self) {
        #[cfg(feature = "trace")]
        let frame_span = ike_util::tracing::info_span!("frame");
        #[cfg(feature = "trace")]
        let _frame_guard = frame_span.enter();

        self.schedule.execute(&mut self.world);

        #[cfg(feature = "tracing-tracy")]
        tracy_client::finish_continuous_frame!();
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);

        self
    }

    pub fn add_event<T: Resource>(&mut self) -> &mut Self {
        self.world.init_resource::<Events<T>>();
        self.add_system_to_stage(Events::<T>::update_system, CoreStage::Start);

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

    pub fn init_resource<T: Resource + FromWorld>(&mut self) -> &mut Self {
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
        label: impl StageLabel,
        before: impl StageLabel,
    ) -> &mut Self {
        self.schedule.add_stage_before(label, before).unwrap();

        self
    }
    /// Adds a new stage before `after`.
    ///
    /// # Panics
    /// - Panics if `after` doesn't exist.
    /// - Panics if `name` already exists.
    pub fn add_stage_after(&mut self, label: impl StageLabel, after: impl StageLabel) -> &mut Self {
        self.schedule.add_stage_after(label, after).unwrap();

        self
    }

    /// Adds a [`System`] to `stage`.
    ///
    /// # Panics
    /// - Panics if `stage` doesn't exist.
    #[track_caller]
    pub fn add_system_to_stage<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
        stage: impl StageLabel,
    ) -> &mut Self {
        self.schedule.add_system_to_stage(system, stage).unwrap();

        self
    }

    /// Adds a [`System`] to [`CoreStage::Update`].
    ///
    /// # Panics
    /// - Panics if [`CoreStage::Update`] doesn't exist.
    pub fn add_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
        self.add_system_to_stage(system, CoreStage::Update);

        self
    }

    pub fn add_startup_system_to_stage<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
        stage: impl StageLabel,
    ) -> &mut Self {
        self.startup.add_system_to_stage(system, stage).unwrap();

        self
    }

    /// Adds a startup [`System`].
    pub fn add_startup_system<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.add_startup_system_to_stage(system, StartupStage::Startup);

        self
    }

    /// Executes startup [`System`]s and runs `self.runner` if present.
    pub fn run(&mut self) {
        self.startup.execute(&mut self.world);

        let mut app = mem::replace(self, App::empty());
        let runner = mem::replace(&mut app.runner, Box::new(RunOnce));
        runner.run(app);
    }
}
