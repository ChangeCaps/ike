use ike_ecs::World;
use ike_math::{Mat4, Vec3};
use ike_transform::GlobalTransform;
use ike_util::tracing::warn;

use crate::{Camera, RawCamera, RenderGraph, SlotValue, Surface, SurfaceError};

fn get_camera(world: &World, aspect: f32) -> RawCamera {
    if let Some((camera, transform)) = world
        .query::<(&Camera, Option<&GlobalTransform>)>()
        .unwrap()
        .iter()
        .next()
    {
        let view = transform.map_or(Mat4::IDENTITY, |transform| transform.matrix());

        let proj = camera.proj_matrix(aspect);

        let position = transform.map_or(Vec3::ZERO, |transform| transform.translation);

        return RawCamera {
            view,
            proj,
            position,
        };
    }

    RawCamera::IDENTITY
}

pub fn render_system(world: &mut World) {
    let mut render_graph = world.remove_resource::<RenderGraph>().unwrap();
    let surface = world.resource::<Surface>();

    let aspect = surface.width() as f32 / surface.height() as f32;

    let surface_texture = match surface.get_current_texture() {
        Ok(surface_texture) => surface_texture,
        Err(SurfaceError::OutOfMemory) => panic!("ran out of vram"),
        Err(err) => {
            warn!("{}", err);

            drop(surface);
            world.insert_resource(render_graph);
            return;
        }
    };

    drop(surface);

    let camera = get_camera(world, aspect);

    let view = surface_texture.create_view(&Default::default());

    render_graph.update(world);
    render_graph
        .run(world, vec![SlotValue::new(view), SlotValue::new(camera)])
        .unwrap();

    #[cfg(feature = "trace")]
    let present_span = ike_util::tracing::info_span!("present");
    #[cfg(feature = "trace")]
    let _present_guard = present_span.enter();

    surface_texture.present();

    world.insert_resource(render_graph);
}