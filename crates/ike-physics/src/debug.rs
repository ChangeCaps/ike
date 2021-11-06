use glam::Vec3;
use ike_core::Query;
use ike_debug_line::DebugLine;
use ike_transform::GlobalTransform;

use crate::BoxCollider;

pub fn debug_box_colliders(query: Query<(&GlobalTransform, &BoxCollider)>) {
    for (transform, collider) in query {
        if let Some(color) = collider.debug {
            let size = transform.scale * collider.size / 2.0;

            DebugLine::new()
                .from(transform * Vec3::new(size.x, size.y, size.z))
                .to(transform * Vec3::new(-size.x, size.y, size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(-size.x, size.y, size.z))
                .to(transform * Vec3::new(-size.x, -size.y, size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(-size.x, -size.y, size.z))
                .to(transform * Vec3::new(size.x, -size.y, size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(size.x, -size.y, size.z))
                .to(transform * Vec3::new(size.x, size.y, size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(size.x, size.y, size.z))
                .to(transform * Vec3::new(size.x, size.y, -size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(-size.x, size.y, size.z))
                .to(transform * Vec3::new(-size.x, size.y, -size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(-size.x, -size.y, size.z))
                .to(transform * Vec3::new(-size.x, -size.y, -size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(size.x, -size.y, size.z))
                .to(transform * Vec3::new(size.x, -size.y, -size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(size.x, size.y, -size.z))
                .to(transform * Vec3::new(-size.x, size.y, -size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(-size.x, size.y, -size.z))
                .to(transform * Vec3::new(-size.x, -size.y, -size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(-size.x, -size.y, -size.z))
                .to(transform * Vec3::new(size.x, -size.y, -size.z))
                .color(color)
                .use_depth()
                .draw();

            DebugLine::new()
                .from(transform * Vec3::new(size.x, -size.y, -size.z))
                .to(transform * Vec3::new(size.x, size.y, -size.z))
                .color(color)
                .use_depth()
                .draw();
        }
    }
}
