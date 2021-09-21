#![deny(unsafe_op_in_unsafe_fn)]

pub mod app;
pub mod avg_luminance_node;
pub mod bloom_node;
pub mod camera;
pub mod color;
pub mod cube_texture;
#[cfg(feature = "2d")]
pub mod d2;
#[cfg(feature = "3d")]
pub mod d3;
#[cfg(feature = "debug")]
pub mod debug;
pub mod editor;
pub mod frame_buffer;
pub mod hdr_pass;
pub mod id;
pub mod input;
pub mod main_pass;
pub mod panels;
pub mod renderer;
#[cfg(feature = "runner")]
pub mod runner;
pub mod state;
#[cfg(feature = "image")]
pub mod texture;
pub mod type_name;
pub mod ui_panel;
pub mod view;
pub mod window;

pub use anyhow;
pub use egui;
pub use ike_wgpu as wgpu;

pub mod prelude {
    pub use crate::app::App;
    pub use crate::avg_luminance_node::AvgLuminanceNode;
    pub use crate::bloom_node::BloomNode;
    pub use crate::camera::{
        Camera, OrthographicProjection, PerspectiveCamera, PerspectiveProjection,
    };
    pub use crate::color::{Color, Color16, Color8};
    pub use crate::cube_texture::{CubeTexture, Environment};
    #[cfg(feature = "2d")]
    pub use crate::d2::{
        font::Font,
        render::{Sprite, SpriteNode2d, TextSprite},
        transform2d::Transform2d,
    };
    #[cfg(feature = "3d")]
    pub use crate::d3::{
        Animation, D3Node, DirectionalLight, Mesh, MeshData, PbrMaterial, PbrMesh, PbrNode,
        PbrScene, PointLight, SkyNode, SkyTexture, Transform3d, Vertex,
    };
    #[cfg(feature = "debug")]
    pub use crate::debug::{DebugLine, DebugMesh, DebugNode};
    pub use crate::export_app;
    pub use crate::frame_buffer::{FrameBuffer, FrameBufferDescriptor};
    pub use crate::hdr_pass::{HdrCombineNode, HdrPass, HdrTarget};
    pub use crate::id::{HasId, Id};
    pub use crate::input::{Input, Mouse};
    pub use crate::main_pass::MainPass;
    pub use crate::panels::{
        inspector_panel::{Inspect, InspectCtx, Inspectable, InspectorPanel},
        view_panel::MainViewPanel,
    };
    pub use crate::renderer::{
        Pass, PassData, PassNode, PassNodeCtx, RenderCtx, RenderNode, RenderNodeCtx, RenderPass,
        Renderer, SampleCount, TargetFormat, TargetSize,
    };
    pub use crate::state::{StartCtx, State, UpdateCtx};
    #[cfg(feature = "image")]
    pub use crate::texture::{
        ColorSpace, HdrTexture, Rgba32Float, Rgba8Unorm, Texture, TextureFormat, TextureVersion,
    };
    pub use crate::ui_panel::UiPanel;
    pub use crate::view::{View, Views};
    pub use crate::wgpu;
    pub use crate::window::Window;
    pub use glam::{swizzles::*, *};
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
