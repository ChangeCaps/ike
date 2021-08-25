use glam::UVec2;

#[derive(Default)]
pub struct Window {
    pub size: UVec2,
    pub fullscreen: bool,
    pub maximized: bool,
    pub cursor_visible: bool,
    pub cursor_grab: bool,
}
