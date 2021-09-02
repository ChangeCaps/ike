pub mod app;
pub mod camera;
pub mod color;
#[cfg(feature = "2d")]
pub mod d2;
#[cfg(feature = "3d")]
pub mod d3;
pub mod editor;
pub mod id;
pub mod input;
pub mod panels;
pub mod renderer;
#[cfg(feature = "runner")]
pub mod runner;
pub mod state;
#[cfg(feature = "image")]
pub mod texture;
pub mod ui_panel;
pub mod view;
pub mod window;

pub use anyhow;
pub use egui;
pub use wgpu;

pub mod prelude {
    pub use crate::app::App;
    pub use crate::camera::{Camera, OrthographicProjection, PerspectiveProjection};
    pub use crate::color::Color;
    #[cfg(feature = "2d")]
    pub use crate::d2::transform2d::Transform2d;
    #[cfg(feature = "3d")]
    pub use crate::d3::{
        mesh::{Mesh, MeshData, Vertex},
        transform3d::Transform3d,
    };
    pub use crate::export_app;
    pub use crate::id::{HasId, Id};
    pub use crate::input;
    pub use crate::panels::{
        inspector_panel::{Inspect, InspectCtx, Inspectable, InspectorPanel},
        view_panel::MainViewPanel,
    };
    pub use crate::renderer::{
        MainPass, PassNode, PassNodeCtx, RenderCtx, RenderPass, SampleCount,
    };
    pub use crate::state::{StartCtx, State, UpdateCtx};
    #[cfg(feature = "image")]
    pub use crate::texture::Texture;
    pub use crate::ui_panel::UiPanel;
    pub use crate::view::{View, Views};
    pub use crate::window::Window;
    pub use glam::{swizzles::*, *};
    pub use wgpu;
    pub use winit::event::{MouseButton, VirtualKeyCode as Key};
}

#[macro_export]
macro_rules! export_app {
    ($app:expr, $state:expr) => {
        mod __ike_extern__ {
            use super::*;

            #[no_mangle]
            fn export_app() -> Box<dyn $crate::app::AppTrait> {
                Box::new($crate::app::AppContainer {
                    app: $app,
                    state: $state,
                })
            }
        }
    };
}
