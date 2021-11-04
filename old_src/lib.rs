#![deny(unsafe_op_in_unsafe_fn)]

pub mod app;
pub mod avg_luminance_node;
pub mod bloom_node;
pub mod camera;
pub mod color;
pub mod cube_texture;
//#[cfg(feature = "2d")]
//pub mod d2;
//pub mod d3;
pub mod buffer;
#[cfg(feature = "debug")]
pub mod debug;
pub mod editor_data;
pub mod frame_buffer;
pub mod hdr_pass;
pub mod input;
pub mod logger;
pub mod panic;
pub mod render_texture;
pub mod renderer;
#[cfg(feature = "runner")]
pub mod runner;
pub mod state;
pub mod texture;
pub mod transform;
pub mod type_name;
pub mod view;
pub mod window;

pub use anyhow;
pub use egui;
pub use ike_wgpu as wgpu;
pub use log;

pub mod prelude {
    pub use crate::app::App;
    pub use crate::avg_luminance_node::LuminanceBufferNode;
    pub use crate::bloom_node::BloomNode;
    pub use crate::buffer::Buffer;
    pub use crate::camera::{
        Camera, OrthographicCamera, OrthographicProjection, PerspectiveCamera,
        PerspectiveProjection,
    };
    pub use crate::color::{Color, Color16, Color8};
    pub use crate::cube_texture::{CubeTexture, Environment};
    pub use crate::transform::Transform;
    /*
    #[cfg(feature = "2d")]
    pub use crate::d2::{
        font::Font,
        render::{Sprite, SpriteNode2d, TextSprite},
        transform2d::Transform2d,
    };
    pub use crate::d3::{
        Animation, D3Node, DirectionalLight, Mesh, MeshData, PbrMaterial, PbrMesh, PbrNode,
        PbrScene, PointLight, SkyNode, Transform3d, Vertex,
    };
    */
    #[cfg(feature = "debug")]
    pub use crate::debug::{DebugLine, DebugNode};
    pub use crate::export_app;
    pub use crate::frame_buffer::{FrameBuffer, FrameBufferDescriptor};
    pub use crate::hdr_pass::HdrCombineNode;
    pub use crate::input::{Input, Mouse};
    pub use crate::renderer::{RenderCtx, RenderGraph, RenderNode};
    pub use crate::state::{EditorCtx, StartCtx, State, UpdateCtx};
    pub use crate::texture::{
        ColorSpace, HdrTexture, Rgba32Float, Rgba8Unorm, Texture, TextureFormat, TextureVersion,
    };
    pub use crate::view::{View, Views};
    pub use crate::wgpu;
    pub use crate::window::Window;
    pub use glam::{swizzles::*, *};
    pub use log::{debug, error, info, trace, warn};
    pub use winit::event::{MouseButton, VirtualKeyCode as Key};
}

#[macro_export]
macro_rules! export_app {
    ($app:expr, $state:expr) => {
        mod __ike_extern__ {
            use super::*;

            #[no_mangle]
            fn export_app(
                hook: Box<dyn Fn(&std::panic::PanicInfo<'_>) + Send + Sync>,
                logger: $crate::logger::Logger,
            ) -> $crate::panic::Result<Box<dyn $crate::app::AppTrait>> {
                std::panic::set_hook(hook);

                let _ = $crate::log::set_boxed_logger(Box::new(logger));

                $crate::panic::catch(std::panic::AssertUnwindSafe(|| {
                    let app: Box<dyn $crate::app::AppTrait> = Box::new($crate::app::AppContainer {
                        app: $app,
                        state: $state,
                    });

                    app
                }))
            }
        }
    };
}
