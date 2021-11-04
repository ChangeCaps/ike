use glam::{Mat4, UVec2, Vec3};
use ike_core::*;
use ike_transform::Transform;

use crate::{
    Camera, MainCamera, PerspectiveProjection, RenderGraph, RenderSurface, RenderTexture,
    ViewInputNode,
};

pub struct RenderSystem;

impl ExclusiveSystem for RenderSystem {
    #[inline]
    fn run(&mut self, world: &mut World) {
        let mut render_graph = world.remove_resource::<RenderGraph>().unwrap();

        let mut render_surface = world.write_resource::<RenderSurface>().unwrap();

        let size = UVec2::new(
            render_surface.config().width,
            render_surface.config().height,
        );

        let samples = 1;
        let format = render_surface.config().format;

        let surface_texture = match render_surface.surface().get_current_texture() {
            Ok(surface_texture) => surface_texture,
            _ => return,
        };

        let render_texture =
            RenderTexture::from_surface_texture(surface_texture, size, samples, format);

        drop(render_surface);

        let main_camera = world.read_resource::<MainCamera>().unwrap();

        let camera = if let Some(camera) = main_camera.0 {
            let node = world.get_node(camera).unwrap();

            let camera = node.get_component::<PerspectiveProjection>().unwrap();

            let (position, view) = if let Some(transform) = node.get_component::<Transform>() {
                (transform.translation, transform.matrix())
            } else {
                (Vec3::ZERO, Mat4::IDENTITY)
            };

            Camera {
                position,
                proj: camera.proj_matrix(),
                view,
            }
        } else {
            Camera {
                position: Vec3::ZERO,
                proj: Mat4::IDENTITY,
                view: Mat4::IDENTITY,
            }
        };

        drop(main_camera);

        let output = render_graph
            .get_output_mut(crate::render_graph::INPUT)
            .unwrap();
        output.set(ViewInputNode::TARGET, render_texture).unwrap();
        output.set(ViewInputNode::CAMERA, camera).unwrap();

        render_graph.update(world);
        render_graph.run(world).unwrap();

        let output = render_graph
            .get_output_mut(crate::render_graph::INPUT)
            .unwrap();
        let render_texture = output
            .remove::<RenderTexture>(ViewInputNode::TARGET)
            .unwrap();

        render_texture.present();

        world.insert_resource(render_graph);
    }
}
