pub struct RenderSettings {
    pub msaa: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self { msaa: 1 }
    }
}
