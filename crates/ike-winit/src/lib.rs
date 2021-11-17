mod resource;
#[cfg(feature = "runner")]
mod runner;

pub use resource::*;
#[cfg(feature = "runner")]
pub use runner::*;

pub type Key = winit::event::VirtualKeyCode;
pub use winit::event::MouseButton;
