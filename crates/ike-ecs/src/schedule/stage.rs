use ike_id::RawLabel;

use crate::{
    ExclusiveSystem, ScheduleError, StageLabel, System, SystemAccess, SystemDescriptor, World,
};

use super::graph;

pub struct ParallelSystem {
    pub(crate) access: SystemAccess,
    pub(crate) labels: Vec<RawLabel>,
    pub(crate) before: Vec<RawLabel>,
    pub(crate) after: Vec<RawLabel>,
    pub(crate) system: Box<dyn System>,
}

impl ParallelSystem {
    pub(crate) fn new(system: impl System) -> Self {
        Self {
            access: system.access(),
            labels: Vec::new(),
            before: Vec::new(),
            after: Vec::new(),
            system: Box::new(system),
        }
    }
}

pub struct ExclusiveSystemDescriptor {
    pub(crate) labels: Vec<RawLabel>,
    pub(crate) before: Vec<RawLabel>,
    pub(crate) after: Vec<RawLabel>,
    pub(crate) system: Box<dyn ExclusiveSystem>,
}

impl ExclusiveSystemDescriptor {
    pub(crate) fn new(system: impl ExclusiveSystem) -> Self {
        Self {
            labels: Vec::new(),
            before: Vec::new(),
            after: Vec::new(),
            system: Box::new(system),
        }
    }
}

/// The schedule contains a list of [`ScheduleStep`]s.
/// Each step contains a collection of [`System`]s all of which have compatible [`SystemAccess`].
pub struct ScheduleStage {
    label: RawLabel,
    parallel_systems: Vec<ParallelSystem>,
    parallel_order: Vec<Vec<usize>>,
    exclusive_systems: Vec<ExclusiveSystemDescriptor>,
    exclusive_order: Vec<usize>,
}

impl ScheduleStage {
    pub fn new(label: impl StageLabel) -> Self {
        Self::new_raw(label.raw_label())
    }

    pub fn new_raw(label: RawLabel) -> Self {
        Self {
            label,
            parallel_systems: Vec::new(),
            parallel_order: Vec::new(),
            exclusive_systems: Vec::new(),
            exclusive_order: Vec::new(),
        }
    }

    pub fn label(&self) -> &RawLabel {
        &self.label
    }

    #[inline]
    pub fn add_system(&mut self, system: SystemDescriptor) -> Result<(), ScheduleError> {
        match system {
            SystemDescriptor::Parallel(parallel) => {
                self.parallel_systems.push(parallel);
            }
            SystemDescriptor::Exclusive(exclusive) => {
                self.exclusive_systems.push(exclusive);
            }
        }

        self.update_execution_order()
    }

    pub fn update_execution_order(&mut self) -> Result<(), ScheduleError> {
        let graph = graph::build_dependency_graph(&self.parallel_systems);
        let order = graph::topological_order(&graph)?;

        self.parallel_order.clear();
        let mut step = Vec::new();
        let mut access = SystemAccess::default();

        for index in order {
            let system = &self.parallel_systems[index];

            if system.access.compatible(&access) {
                access.combine(system.access.clone());

                step.push(index);
            } else {
                access = SystemAccess::default();

                self.parallel_order.push(step);
                step = vec![index];
            }
        }

        self.parallel_order.push(step);

        debug_assert!(self.validate_parallel_order());

        let graph = graph::build_dependency_graph(&self.exclusive_systems);
        self.exclusive_order = graph::topological_order(&graph)?;

        Ok(())
    }

    pub fn validate_parallel_order(&self) -> bool {
        let mut seen_indices = Vec::new();

        for step in &self.parallel_order {
            for index in step {
                if seen_indices.contains(index) {
                    return false;
                } else {
                    seen_indices.push(*index);
                }
            }
        }

        true
    }

    /// Executes all step in sequence, the systems within each step is executed in parallel if
    /// the `rayon` feature is enabled, in sequence if not.
    #[inline]
    pub fn execute(&mut self, world: &mut World) {
        for &index in &self.exclusive_order {
            self.exclusive_systems[index].system.run(world);
        }

        let parallel_systems = self.parallel_systems.as_mut_ptr();

        for step in &self.parallel_order {
            for &index in step {
                let system = unsafe { &mut *parallel_systems.add(index) };
                system.system.init(world);
            }

            for &index in step {
                let system = unsafe { &mut *parallel_systems.add(index) };
                system.system.run(world);
            }

            for &index in step {
                let system = unsafe { &mut *parallel_systems.add(index) };
                system.system.apply(world);
            }
        }
    }
}
