use std::process::Command;

pub enum BuildMode {
    Debug,
    Release,
}

#[derive(Default, Debug)]
pub struct BuildCommand {
    pub cargo_args: Vec<String>,
    pub rustc_args: Vec<String>,
}

impl BuildCommand {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn cargo_arg(&mut self, arg: impl Into<String>) {
        self.cargo_args.push(arg.into());
    }

    #[inline]
    pub fn rustc_arg(&mut self, arg: impl Into<String>) {
        self.rustc_args.push(arg.into());
    }

    #[inline]
    pub fn cfg(&mut self, cfg: impl Into<String>) {
        self.rustc_arg("--cfg");
        self.rustc_arg(cfg);
    }

    #[inline]
    pub fn manifest_path(&mut self, path: impl Into<String>) {
        self.cargo_arg("--manifest-path");
        self.cargo_arg(path);
    }

    #[inline]
    pub fn build_mode(&mut self, mode: BuildMode) {
        match mode {
            BuildMode::Release => self.cargo_arg("--release"),
            BuildMode::Debug => {}
        }
    }

    #[inline]
    pub fn crate_type(&mut self, crate_type: impl Into<String>) {
        self.rustc_arg("--crate-type");
        self.rustc_arg(crate_type);
    }

    #[inline]
    pub fn command(self) -> Command {
        let mut command = Command::new("cargo");
        command.arg("rustc");

        for cargo_arg in self.cargo_args {
            command.arg(cargo_arg);
        }

        command.arg("--");

        for rustc_arg in self.rustc_args {
            command.arg(rustc_arg);
        }

        command
    }
}

impl std::fmt::Display for BuildCommand {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cargo rustc")?;

        for cargo_arg in &self.cargo_args {
            write!(f, " {}", cargo_arg)?;
        }

        write!(f, " --")?;

        for rustc_arg in &self.rustc_args {
            write!(f, " {}", rustc_arg)?;
        }

        Ok(())
    }
}
