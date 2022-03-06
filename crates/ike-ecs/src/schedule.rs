use std::{any::TypeId, collections::BTreeMap};

use crate::{ExclusiveSystem, System, SystemAccess, World};

/// A step contains a collection of [`System`]s with compatible [`SystemAccess`].
#[derive(Default)]
pub struct ScheduleStep {
    pub access: SystemAccess,
    pub systems: Vec<Box<dyn System>>,
}

/// The schedule contains a list of [`ScheduleStep`]s.
/// Each step contains a collection of [`System`]s all of which have compatible [`SystemAccess`].
#[derive(Default)]
pub struct Schedule {
    steps: Vec<ScheduleStep>,
    exclusive_systems: BTreeMap<TypeId, Box<dyn ExclusiveSystem>>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            exclusive_systems: BTreeMap::new(),
        }
    }

    #[inline]
    pub fn add_system<T: System>(&mut self, system: T) {
        let access = system.access();

        for step in &mut self.steps {
            if step.access.compatible(&access) {
                step.access.combine(access);

                step.systems.push(Box::new(system));

                return;
            }
        }

        let mut step = ScheduleStep {
            access,
            systems: Vec::new(),
        };

        step.systems.push(Box::new(system));

        self.steps.push(step);
    }

    #[inline]
    pub fn add_exclusive_system<T: ExclusiveSystem>(&mut self, system: T) {
        self.exclusive_systems
            .insert(TypeId::of::<T>(), Box::new(system));
    }

    /// Executes all step in sequence, the systems within each step is executed in parallel if
    /// the `rayon` feature is enabled, in sequence if not.
    #[inline]
    pub fn execute(&mut self, world: &mut World) {
        for system in self.exclusive_systems.values_mut() {
            system.run(world);
        }

        for step in &mut self.steps {
            for system in &mut step.systems {
                system.init(world);
            }

            #[cfg(not(feature = "rayon"))]
            step.systems.iter_mut().for_each(|system| {
                system.run(world);
            });

            for system in &mut step.systems {
                system.apply(world);
            }
        }
    }
}
