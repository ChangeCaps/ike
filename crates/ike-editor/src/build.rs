use std::process::{Child, Command};

use ike::renderer::RenderCtx;

use crate::{logger::LogReceiver, panic::Panics, EditorState};

#[derive(Debug)]
pub enum BuildState {
    Unloaded,
    Building { process: Child },
    Loaded,
}

impl Default for BuildState {
    #[inline]
    fn default() -> Self {
        Self::Unloaded
    }
}

impl EditorState {
    pub fn building(&self) -> bool {
        match self.build_state {
            BuildState::Building { .. } => true,
            _ => false,
        }
    }

    pub fn build(&mut self) -> ike::anyhow::Result<()> {
        if let BuildState::Loaded = self.build_state {
            self.unload();
        }

        if let BuildState::Building { ref mut process } = self.build_state {
            process.kill()?;
        }

        let mut command = Command::new("cargo");
        command
            .arg("build")
            .arg("--manifest-path")
            .arg(self.project.manifest_path(&self.path))
            .arg("--target-dir")
            .arg(self.project.target_path(&self.path))
            .arg("--release");

        let child = command.spawn()?;

        self.build_state = BuildState::Building { process: child };

        Ok(())
    }

    pub fn update_build(
        &mut self,
        panics: &Panics,
        logger: &LogReceiver,
        render_ctx: &RenderCtx,
    ) -> ike::anyhow::Result<()> {
        if let BuildState::Building { ref mut process } = self.build_state {
            if let Some(exit) = process.try_wait()? {
                if !exit.success() {
                    self.build_state = BuildState::Unloaded;

                    return Ok(());
                }

                self.load(
                    panics,
                    logger,
                    render_ctx,
                    self.project
                        .target_path(&self.path)
                        .join("release")
                        .join("ike_example.dll"),
                )?;

                self.build_state = BuildState::Loaded;
            }
        }

        Ok(())
    }
}
