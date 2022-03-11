use ike_math::{Vec2, Vec3};

use crate::{Mesh, MeshTool};

impl Mesh {
    pub fn cube(size: Vec3) -> Self {
        fn map(normal: Vec3, pos: Vec2) -> Vec3 {
            Vec3::new(1.0, pos.y, pos.x) * normal.x
                + Vec3::new(pos.x, 1.0, pos.y) * normal.y
                + Vec3::new(pos.y, pos.x, 1.0) * normal.z
        }

        fn side(mesh_tool: &mut MeshTool, normal: Vec3, size: Vec3) {
            mesh_tool.push_vertex(map(normal * size, Vec2::new(-1.0, -1.0)));
            mesh_tool.set_normal(normal);
            mesh_tool.set_uv_0(Vec2::new(0.0, 0.0));

            mesh_tool.push_vertex(map(normal * size, Vec2::new(1.0, -1.0)));
            mesh_tool.set_normal(normal);
            mesh_tool.set_uv_0(Vec2::new(1.0, 0.0));

            mesh_tool.push_vertex(map(normal * size, Vec2::new(-1.0, 1.0)));
            mesh_tool.set_normal(normal);
            mesh_tool.set_uv_0(Vec2::new(0.0, 1.0));

            mesh_tool.push_vertex(map(normal * size, Vec2::new(1.0, 1.0)));
            mesh_tool.set_normal(normal);
            mesh_tool.set_uv_0(Vec2::new(1.0, 1.0));
        }

        let mut mesh_tool = MeshTool::new();

        side(&mut mesh_tool, Vec3::X, size / 2.0);
        side(&mut mesh_tool, Vec3::Y, size / 2.0);
        side(&mut mesh_tool, Vec3::Z, size / 2.0);
        side(&mut mesh_tool, -Vec3::X, size / 2.0);
        side(&mut mesh_tool, -Vec3::Y, size / 2.0);
        side(&mut mesh_tool, -Vec3::Z, size / 2.0);

        for i in 0..3 {
            mesh_tool.push_index(i * 4 + 0);
            mesh_tool.push_index(i * 4 + 2);
            mesh_tool.push_index(i * 4 + 1);

            mesh_tool.push_index(i * 4 + 1);
            mesh_tool.push_index(i * 4 + 2);
            mesh_tool.push_index(i * 4 + 3);
        }

        for i in 3..6 {
            mesh_tool.push_index(i * 4 + 0);
            mesh_tool.push_index(i * 4 + 1);
            mesh_tool.push_index(i * 4 + 2);

            mesh_tool.push_index(i * 4 + 1);
            mesh_tool.push_index(i * 4 + 3);
            mesh_tool.push_index(i * 4 + 2);
        }

        let mut mesh = mesh_tool.finish();
        mesh.calculate_tangents();
        mesh
    }
}
