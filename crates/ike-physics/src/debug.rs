use ike_debug::DebugLine;
use ike_ecs::Query;
use ike_math::{const_vec3, Vec3};
use ike_transform::GlobalTransform;

use crate::{BoxCollider, DebugCollider};

const EDGES: [[Vec3; 2]; 12] = [
    [const_vec3!([1.0, 1.0, 1.0]), const_vec3!([1.0, 1.0, -1.0])],
    [const_vec3!([1.0, 1.0, 1.0]), const_vec3!([1.0, -1.0, 1.0])],
    [const_vec3!([1.0, 1.0, 1.0]), const_vec3!([-1.0, 1.0, 1.0])],
    [
        const_vec3!([-1.0, -1.0, -1.0]),
        const_vec3!([-1.0, -1.0, 1.0]),
    ],
    [
        const_vec3!([-1.0, -1.0, -1.0]),
        const_vec3!([-1.0, 1.0, -1.0]),
    ],
    [
        const_vec3!([-1.0, -1.0, -1.0]),
        const_vec3!([1.0, -1.0, -1.0]),
    ],
    [
        const_vec3!([-1.0, -1.0, 1.0]),
        const_vec3!([-1.0, 1.0, 1.0]),
    ],
    [
        const_vec3!([-1.0, -1.0, 1.0]),
        const_vec3!([1.0, -1.0, 1.0]),
    ],
    [
        const_vec3!([1.0, -1.0, -1.0]),
        const_vec3!([1.0, 1.0, -1.0]),
    ],
    [
        const_vec3!([1.0, -1.0, -1.0]),
        const_vec3!([1.0, -1.0, 1.0]),
    ],
    [
        const_vec3!([-1.0, 1.0, -1.0]),
        const_vec3!([1.0, 1.0, -1.0]),
    ],
    [
        const_vec3!([-1.0, 1.0, -1.0]),
        const_vec3!([-1.0, 1.0, 1.0]),
    ],
];

pub fn debug_box_collider_system(query: Query<(&BoxCollider, &GlobalTransform, &DebugCollider)>) {
    for (box_collider, global_transform, _debug_collider) in query.iter() {
        let size = box_collider.size * global_transform.scale / 2.0;

        for [from, to] in EDGES {
            DebugLine::new(
                global_transform.transform() * (from * size),
                global_transform.transform() * (to * size),
            )
            .draw();
        }
    }
}
