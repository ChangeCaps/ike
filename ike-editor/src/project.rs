use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer, Serialize};

fn process_path<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut path = <Option<PathBuf>>::deserialize(deserializer)?;

    if let Some(path) = &mut path {
        *path = path.iter().collect();
    }

    Ok(path)
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    #[serde(deserialize_with = "process_path")]
    pub manifest_path: Option<PathBuf>,
    #[serde(deserialize_with = "process_path")]
    pub target_path: Option<PathBuf>,
}

impl Project {
    #[inline]
    pub fn load(path: &Path) -> ike::anyhow::Result<Self> {
        if let Ok(toml_string) = read_to_string(path) {
            Ok(toml::from_str(&toml_string)?)
        } else {
            Ok(Self::default())
        }
    }

    #[inline]
    pub fn manifest_path(&self, path: &Path) -> PathBuf {
        if let Some(manifest_path) = &self.manifest_path {
            path.join(manifest_path)
        } else {
            path.join("Cargo.toml")
        }
    }

    #[inline]
    pub fn target_path(&self, path: &Path) -> PathBuf {
        if let Some(target_path) = &self.target_path {
            path.join(target_path)
        } else {
            path.join("Cargo.toml")
        }
    }
}
