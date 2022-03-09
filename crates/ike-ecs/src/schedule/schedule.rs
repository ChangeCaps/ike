use std::collections::HashSet;

use ike_id::RawLabel;

use crate::{IntoSystemDescriptor, ScheduleStage, StageLabel, World};

#[derive(Default)]
pub struct Schedule {
    stages: Vec<ScheduleStage>,
}

impl Schedule {
    pub const fn new() -> Self {
        Self { stages: Vec::new() }
    }

    fn stage_index(&self, label: &RawLabel) -> Option<usize> {
        self.stages.iter().position(|stage| stage.label() == label)
    }

    fn contains(&self, label: &RawLabel) -> bool {
        self.stage_index(label).is_some()
    }

    pub fn push_stage(&mut self, label: impl StageLabel) -> Result<(), ScheduleError> {
        let label = label.raw_label();

        if self.contains(&label) {
            return Err(ScheduleError::AlreadyContainsStage(label));
        }

        self.stages.push(ScheduleStage::new_raw(label));

        return Ok(());
    }

    pub fn add_stage_before(
        &mut self,
        label: impl StageLabel,
        before: impl StageLabel,
    ) -> Result<(), ScheduleError> {
        let label = label.raw_label();
        let before = before.raw_label();

        if self.contains(&label) {
            return Err(ScheduleError::AlreadyContainsStage(label));
        }

        let index = self
            .stage_index(&before)
            .ok_or(ScheduleError::InvalidStage(before))?;

        self.stages.insert(index, ScheduleStage::new_raw(label));

        Ok(())
    }

    pub fn add_stage_after(
        &mut self,
        label: impl StageLabel,
        after: impl StageLabel,
    ) -> Result<(), ScheduleError> {
        let label = label.raw_label();
        let before = after.raw_label();

        if self.contains(&label) {
            return Err(ScheduleError::AlreadyContainsStage(label));
        }

        let index = self
            .stage_index(&before)
            .ok_or(ScheduleError::InvalidStage(before))?;

        self.stages.insert(index + 1, ScheduleStage::new_raw(label));

        Ok(())
    }

    pub fn add_system_to_stage<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
        stage: impl StageLabel,
    ) -> Result<(), ScheduleError> {
        let stage = stage.raw_label();

        let index = self
            .stage_index(&stage)
            .ok_or(ScheduleError::InvalidStage(stage))?;

        self.stages[index].add_system(system.into_descriptor())
    }

    pub fn execute(&mut self, world: &mut World) {
        for stage in &mut self.stages {
            stage.execute(world);
        }
    }

    pub fn iter_stages(&self) -> impl Iterator<Item = &ScheduleStage> {
        self.stages.iter()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ScheduleError {
    #[error("stage '{0:?}' does not exist")]
    InvalidStage(RawLabel),
    #[error("already contains stage '{0:?}'")]
    AlreadyContainsStage(RawLabel),
    #[error("found dependency cycle in {0:?}")]
    GraphCycles(Vec<(usize, HashSet<RawLabel>)>),
}
