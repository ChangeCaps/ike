use ike_egui::egui;
use std::{io, path::Path};

use crate::scenes::Scenes;

#[derive(Default)]
pub struct FileBrowser {}

impl FileBrowser {
    #[inline]
    pub fn ui(&mut self, ui: &mut egui::Ui, scenes: &mut Scenes) -> io::Result<()> {
        self.ui_dir(ui, scenes, &Path::new("."))
    }

    #[inline]
    fn ui_dir(&mut self, ui: &mut egui::Ui, scenes: &mut Scenes, path: &Path) -> io::Result<()> {
        let mut paths = Vec::new();

        for entry in path.read_dir()? {
            let entry = entry?;

            let path = entry.path();

            paths.push(path);
        }

        paths.retain(|path| {
            if path.is_dir() {
                self.ui_path(ui, scenes, path).unwrap();

                false
            } else {
                true
            }
        });

        for path in paths {
            self.ui_path(ui, scenes, &path)?;
        }

        Ok(())
    }

    #[inline]
    fn ui_path(&mut self, ui: &mut egui::Ui, scenes: &mut Scenes, path: &Path) -> io::Result<()> {
        let name = path.file_name().unwrap().to_str().unwrap();

        if path.is_dir() {
            let res: Option<io::Result<()>> = ui
                .collapsing(name, |ui| self.ui_dir(ui, scenes, path))
                .body_returned;

            if let Some(res) = res {
                res?;
            }
        } else {
            let ext = path.extension().map(|ext| ext.to_str().unwrap());

			let name = match ext {
				Some("scn") => format!("{} 🎬", name),
				_ => format!("{}", name),
			};

            if ui.button(name).clicked() {
				match ext {
					Some("scn") if !scenes.scene_loaded(path) => {
						scenes.load_scene(path);
					}
					_ => {}
				}
			}
        }

        Ok(())
    }
}
