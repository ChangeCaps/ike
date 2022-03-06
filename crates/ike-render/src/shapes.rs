use ike_math::{Vec2, Vec3};

use crate::Mesh;

impl Mesh {
    pub fn cube(size: Vec3) -> Self {
        fn map(normal: Vec3, pos: Vec2) -> Vec3 {
            Vec3::new(1.0, pos.x, pos.y) * normal.x
                + Vec3::new(pos.x, 1.0, pos.y) * normal.y
                + Vec3::new(pos.x, pos.y, 1.0) * normal.z
        }

        fn side(positions: &mut Vec<Vec3>, normals: &mut Vec<Vec3>, normal: Vec3, size: Vec3) {
            positions.push(map(normal * size, Vec2::new(-1.0, -1.0)));
            positions.push(map(normal * size, Vec2::new(1.0, -1.0)));
            positions.push(map(normal * size, Vec2::new(-1.0, 1.0)));
            positions.push(map(normal * size, Vec2::new(1.0, 1.0)));

            normals.push(normal);
            normals.push(normal);
            normals.push(normal);
            normals.push(normal);
        }

        let mut positions = Vec::new();
        let mut normals = Vec::new();

        side(&mut positions, &mut normals, Vec3::X, size / 2.0);
        side(&mut positions, &mut normals, -Vec3::X, size / 2.0);
        side(&mut positions, &mut normals, Vec3::Y, size / 2.0);
        side(&mut positions, &mut normals, -Vec3::Y, size / 2.0);
        side(&mut positions, &mut normals, Vec3::Z, size / 2.0);
        side(&mut positions, &mut normals, -Vec3::Z, size / 2.0);

        let mut mesh = Self::new();

        mesh.insert_attribute(Mesh::POSITION, positions);
        mesh.insert_attribute(Mesh::NORMAL, normals);

        let indices = mesh.get_indices_mut();

        for i in 0..6 {
            indices.push(i * 4 + 0);
            indices.push(i * 4 + 1);
            indices.push(i * 4 + 2);

            indices.push(i * 4 + 1);
            indices.push(i * 4 + 2);
            indices.push(i * 4 + 3);
        }

        mesh
    }
}
