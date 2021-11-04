use std::collections::HashSet;

use crate::render_device;

pub struct Shader<'a> {
    wgsl: &'a str,
    defs: HashSet<&'a str>,
}

impl<'a> Shader<'a> {
    #[inline]
    pub fn new(wgsl: &'a str) -> Self {
        Self {
            wgsl,
            defs: HashSet::new(),
        }
    }

    #[inline]
    pub fn set(&mut self, def: &'a str) {
        self.defs.insert(def);
    }

    #[inline]
    pub fn remove(&mut self, def: &'a str) {
        self.defs.remove(def);
    }

    #[inline]
    pub fn get_wgsl(&self) -> &'a str {
        self.wgsl
    }

    #[inline]
    pub fn processed_wgsl(&self) -> Option<String> {
        let mut old_wgsl = self.wgsl;

        let mut wgsl = String::new();

        while let Some(idx) = old_wgsl.find("#ifdef") {
            let (left, right) = old_wgsl.split_at(idx);
            let (_, right) = right.split_at(6);

            wgsl += left;

            let idx = right.find(|c: char| !c.is_whitespace())?;

            let (_, right) = right.split_at(idx);

            let idx = right.find(|c: char| c.is_whitespace())?;

            let (def, right) = right.split_at(idx);

            let idx = right.find("#endif")?;

            let (code, right) = right.split_at(idx);

            if self.defs.contains(def) {
                wgsl += code;
            }

            let (_, right) = right.split_at(6);

            old_wgsl = right;
        }

        wgsl += old_wgsl;

        Some(wgsl)
    }

    #[inline]
    pub fn get_module(&self) -> Option<ike_wgpu::ShaderModule> {
        let process = self.processed_wgsl()?;

        let module = render_device().create_shader_module(&ike_wgpu::ShaderModuleDescriptor {
            label: None,
            source: ike_wgpu::ShaderSource::Wgsl(process.into()),
        });

        Some(module)
    }
}
