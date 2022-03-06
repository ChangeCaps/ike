use std::{
    error::Error,
    fmt::{Debug, Display},
};

use ike_ecs::{ExclusiveSystem, Schedule, System, World};

pub struct AppStage {
    name: String,
    schedule: Schedule,
}

impl AppStage {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            schedule: Schedule::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// TODO(Hjalte): move this into ike-ecs and rename to SystemStages
#[derive(Default)]
pub struct AppStages {
    stages: Vec<AppStage>,
}

impl AppStages {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn contains_stage(&self, name: impl AsRef<str>) -> bool {
        self.stages
            .iter()
            .find(|stage| stage.name == name.as_ref())
            .is_some()
    }

    pub fn stage_position(&self, name: impl AsRef<str>) -> Option<usize> {
        self.stages
            .iter()
            .position(|stage| stage.name == name.as_ref())
    }

    pub fn get_stage(&self, name: impl AsRef<str>) -> Option<&AppStage> {
        self.stages.iter().find(|stage| stage.name == name.as_ref())
    }

    pub fn get_stage_mut(&mut self, name: impl AsRef<str>) -> Option<&mut AppStage> {
        self.stages
            .iter_mut()
            .find(|stage| stage.name == name.as_ref())
    }

    pub fn push_stage(&mut self, name: impl Into<String>) -> Result<(), AppStagesError> {
        let name = name.into();

        if !self.contains_stage(&name) {
            self.stages.push(AppStage::new(name));

            Ok(())
        } else {
            Err(AppStagesError::StageAlreadyExists(name))
        }
    }

    #[track_caller]
    pub fn insert_stage_before(
        &mut self,
        name: impl Into<String>,
        before: impl AsRef<str>,
    ) -> Result<(), AppStagesError> {
        let name = name.into();

        if self.contains_stage(&name) {
            return Err(AppStagesError::StageAlreadyExists(name));
        }

        let position = self
            .stage_position(&before)
            .ok_or_else(|| AppStagesError::StageNotFound(before.as_ref().to_string()))?;

        self.stages.insert(position, AppStage::new(name));

        Ok(())
    }

    #[track_caller]
    pub fn insert_stage_after(
        &mut self,
        name: impl Into<String>,
        after: impl AsRef<str>,
    ) -> Result<(), AppStagesError> {
        let name = name.into();

        if self.contains_stage(&name) {
            return Err(AppStagesError::StageAlreadyExists(name));
        }

        let position = self
            .stage_position(&after)
            .ok_or_else(|| AppStagesError::StageNotFound(after.as_ref().to_string()))?;

        self.stages.insert(position + 1, AppStage::new(name));

        Ok(())
    }

    pub fn add_system_to_stage(
        &mut self,
        system: impl System,
        stage: impl AsRef<str>,
    ) -> Result<(), AppStagesError> {
        let stage = self
            .get_stage_mut(&stage)
            .ok_or_else(|| AppStagesError::StageNotFound(stage.as_ref().to_string()))?;

        stage.schedule.add_system(system);

        Ok(())
    }

    pub fn add_exclusive_system_to_stage(
        &mut self,
        system: impl ExclusiveSystem,
        stage: impl AsRef<str>,
    ) -> Result<(), AppStagesError> {
        let stage = self
            .get_stage_mut(&stage)
            .ok_or_else(|| AppStagesError::StageNotFound(stage.as_ref().to_string()))?;

        stage.schedule.add_exclusive_system(system);

        Ok(())
    }

    pub fn iter_stages(&self) -> impl Iterator<Item = &AppStage> {
        self.stages.iter()
    }

    pub fn execute(&mut self, world: &mut World) {
        for stage in &mut self.stages {
            stage.schedule.execute(world);
        }
    }
}

#[derive(Debug)]
pub enum AppStagesError {
    StageAlreadyExists(String),
    StageNotFound(String),
}

impl Display for AppStagesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StageAlreadyExists(name) => {
                write!(f, "stage '{}' already exists", name)
            }
            Self::StageNotFound(name) => {
                write!(f, "stage '{}' not found", name)
            }
        }
    }
}

impl Error for AppStagesError {}
