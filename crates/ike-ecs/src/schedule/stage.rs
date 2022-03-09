use std::collections::HashSet;

use ike_id::RawLabel;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

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
        let mut step_dependencies = HashSet::new();

        for index in order {
            let system = &self.parallel_systems[index];
            let system_dependencies = &graph[&index];

            let dependencies_incompatible = system_dependencies
                .keys()
                .map(|index| step_dependencies.insert(*index))
                .fold(false, |acc, val| acc || val);

            if access.compatible(&system.access) && !dependencies_incompatible {
                access.combine(system.access.clone());

                step.push(index);
            } else {
                access = system.access.clone();
                step_dependencies = system_dependencies.keys().cloned().collect();

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

        struct Ptr(*mut ParallelSystem);

        impl Ptr {
            pub unsafe fn get_mut(&self, index: usize) -> &mut ParallelSystem {
                unsafe { &mut *self.0.add(index) }
            }
        }

        unsafe impl Send for Ptr {}
        unsafe impl Sync for Ptr {}

        let parallel_systems = Ptr(self.parallel_systems.as_mut_ptr());

        for step in &self.parallel_order {
            for &index in step {
                let system = unsafe { parallel_systems.get_mut(index) };
                system.system.init(world);
            }

            #[cfg(feature = "rayon")]
            step.par_iter().for_each(|index| {
                let system = unsafe { parallel_systems.get_mut(*index) };
                system.system.run(world);
            });

            #[cfg(not(feature = "rayon"))]
            for &index in step {
                let system = unsafe { parallel_systems.get_mut(index) };
                system.system.run(world);
            }

            for &index in step {
                let system = unsafe { parallel_systems.get_mut(index) };
                system.system.apply(world);
            }
        }
    }
}
