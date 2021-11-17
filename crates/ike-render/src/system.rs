use glam::{Mat4, UVec2, Vec3};
use ike_core::*;
use ike_transform::GlobalTransform;

use crate::{
    Camera, MainCamera, PerspectiveProjection, RenderGraph, RenderSurface, RenderTexture,
    ViewInputNode,
};

pub fn render_graph_update_system(world: WorldRef) {
    let mut render_graph = world.get_resource_mut::<RenderGraph>().unwrap();

    render_graph.update(&world);
}

pub fn camera_aspect_system(
    render_surface: Res<RenderSurface>,
    query: Query<&mut PerspectiveProjection>,
) {
    let aspect = render_surface.config().width as f32 / render_surface.config().height as f32;

    for mut projection in query {
        projection.aspect = aspect;
    }
}

pub fn render_system(world: WorldRef) {
    let mut render_graph = world.get_resource_mut::<RenderGraph>().unwrap();

    let mut render_surface = world.get_resource_mut::<RenderSurface>().unwrap();

    let size = UVec2::new(
        render_surface.config().width,
        render_surface.config().height,
    );

    let samples = 1;
    let format = render_surface.config().format;

    let surface_texture = match render_surface.next_frame() {
        Ok(surface_texture) => surface_texture,
        _ => return,
    };

    let view = surface_texture.texture().create_view(&Default::default());

    let render_texture = RenderTexture::new(view, size, samples, format);

    let mut main_camera = world.query_filter::<Entity, With<MainCamera>>().unwrap();

    let camera = if let Some(camera) = main_camera.next() {
        let proj = world
            .get_component::<PerspectiveProjection>(&camera)
            .unwrap();

        let transform = world.get_component::<GlobalTransform>(&camera);

        let (position, view) = if let Some(transform) = transform {
            (transform.translation, transform.matrix())
        } else {
            (Vec3::ZERO, Mat4::IDENTITY)
        };

        Camera {
            position,
            proj: proj.proj_matrix(),
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

    render_graph.run(&world).unwrap();

    surface_texture.present();
}
