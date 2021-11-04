use glam::{Vec2, Vec3};

use crate::{Color, Mesh};

impl Mesh {
    #[inline]
    pub fn cube(mut size: Vec3) -> Self {
        size /= 2.0;

        let mut mesh = Self::new();

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut colors = Vec::new();
        let mut uvs = Vec::new();
        let mut tangents = Vec::new();

        positions.push(Vec3::new(-size.x, size.y, -size.z));
        normals.push(Vec3::Y);
        tangents.push(Vec3::Z);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 0.0));

        positions.push(Vec3::new(size.x, size.y, -size.z));
        normals.push(Vec3::Y);
        tangents.push(Vec3::Z);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 0.0));

        positions.push(Vec3::new(-size.x, size.y, size.z));
        normals.push(Vec3::Y);
        tangents.push(Vec3::Z);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 1.0));

        positions.push(Vec3::new(size.x, size.y, size.z));
        normals.push(Vec3::Y);
        tangents.push(Vec3::Z);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 1.0));

        positions.push(Vec3::new(-size.x, -size.y, -size.z));
        normals.push(-Vec3::Y);
        tangents.push(-Vec3::Z);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 0.0));

        positions.push(Vec3::new(size.x, -size.y, -size.z));
        normals.push(-Vec3::Y);
        tangents.push(-Vec3::Z);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 0.0));

        positions.push(Vec3::new(-size.x, -size.y, size.z));
        normals.push(-Vec3::Y);
        tangents.push(-Vec3::Z);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 1.0));

        positions.push(Vec3::new(size.x, -size.y, size.z));
        normals.push(-Vec3::Y);
        tangents.push(-Vec3::Z);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 1.0));

        positions.push(Vec3::new(size.x, -size.y, -size.z));
        normals.push(Vec3::X);
        tangents.push(Vec3::Y);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 0.0));

        positions.push(Vec3::new(size.x, -size.y, size.z));
        normals.push(Vec3::X);
        tangents.push(Vec3::Y);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 0.0));

        positions.push(Vec3::new(size.x, size.y, -size.z));
        normals.push(Vec3::X);
        tangents.push(Vec3::Y);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 1.0));

        positions.push(Vec3::new(size.x, size.y, size.z));
        normals.push(Vec3::X);
        tangents.push(Vec3::Y);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 1.0));

        positions.push(Vec3::new(-size.x, -size.y, -size.z));
        normals.push(-Vec3::X);
        tangents.push(-Vec3::Y);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 0.0));

        positions.push(Vec3::new(-size.x, -size.y, size.z));
        normals.push(-Vec3::X);
        tangents.push(-Vec3::Y);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 0.0));

        positions.push(Vec3::new(-size.x, size.y, -size.z));
        normals.push(-Vec3::X);
        tangents.push(-Vec3::Y);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 1.0));

        positions.push(Vec3::new(-size.x, size.y, size.z));
        normals.push(-Vec3::X);
        tangents.push(-Vec3::Y);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 1.0));

        positions.push(Vec3::new(-size.x, -size.y, size.z));
        normals.push(Vec3::Z);
        tangents.push(Vec3::X);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 0.0));

        positions.push(Vec3::new(-size.x, size.y, size.z));
        normals.push(Vec3::Z);
        tangents.push(Vec3::X);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 0.0));

        positions.push(Vec3::new(size.x, -size.y, size.z));
        normals.push(Vec3::Z);
        tangents.push(Vec3::X);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 1.0));

        positions.push(Vec3::new(size.x, size.y, size.z));
        normals.push(Vec3::Z);
        tangents.push(Vec3::X);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 1.0));

        positions.push(Vec3::new(-size.x, -size.y, -size.z));
        normals.push(-Vec3::Z);
        tangents.push(-Vec3::X);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 0.0));

        positions.push(Vec3::new(-size.x, size.y, -size.z));
        normals.push(-Vec3::Z);
        tangents.push(-Vec3::X);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 0.0));

        positions.push(Vec3::new(size.x, -size.y, -size.z));
        normals.push(-Vec3::Z);
        tangents.push(-Vec3::X);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(0.0, 1.0));

        positions.push(Vec3::new(size.x, size.y, -size.z));
        normals.push(-Vec3::Z);
        tangents.push(-Vec3::X);
        colors.push(Color::WHITE);
        uvs.push(Vec2::new(1.0, 1.0));

        mesh.insert(Mesh::POSITION, positions);
        mesh.insert(Mesh::NORMAL, normals);
        mesh.insert(Mesh::TANGENT, tangents);
        mesh.insert(Mesh::COLOR, colors);
        mesh.insert(Mesh::UV, uvs);

        let indices = mesh.indices_mut();

        indices.push(0);
        indices.push(2);
        indices.push(1);
        indices.push(1);
        indices.push(2);
        indices.push(3);

        indices.push(4);
        indices.push(5);
        indices.push(6);
        indices.push(6);
        indices.push(5);
        indices.push(7);

        indices.push(8);
        indices.push(10);
        indices.push(9);
        indices.push(9);
        indices.push(10);
        indices.push(11);

        indices.push(12);
        indices.push(13);
        indices.push(14);
        indices.push(14);
        indices.push(13);
        indices.push(15);

        indices.push(16);
        indices.push(18);
        indices.push(17);
        indices.push(17);
        indices.push(18);
        indices.push(19);

        indices.push(20);
        indices.push(21);
        indices.push(22);
        indices.push(22);
        indices.push(21);
        indices.push(23);

        mesh
    }

    /*
    /// Create quad on the xy plane.
    #[inline]
    pub fn quad(size: Vec2) -> Self {
        let mut mesh = Self::new();


            positions.push( Vec3::new(-size.x, -size.y, 0.0));
            normals.push(Vec3::Z);
            tangents.push(Vec4::Y);
            colors.push(Color::WHITE);
            uvs.push(Vec2::ZERO);




            positions.push( Vec3::new(size.x, -size.y, 0.0));
            normals.push(Vec3::Z);
            tangents.push(Vec4::Y);
            colors.push(Color::WHITE);
            uvs.push(Vec2::new(1.0); 0.0));




            positions.push( Vec3::new(-size.x, size.y, 0.0));
            normals.push(Vec3::Z);
            tangents.push(Vec4::Y);
            colors.push(Color::WHITE);
            uvs.push(Vec2::new(0.0); 1.0));




            positions.push( Vec3::new(size.x, size.y, 0.0));
            normals.push(Vec3::Z);
            tangents.push(Vec4::Y);
            colors.push(Color::WHITE);
            uvs.push(Vec2::ONE);



        mesh.indices.push(0);
        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(3);

        mesh
    }

    #[inline]
    pub fn plane(size: Vec2) -> Self {
        let mut mesh = Self::new();


            positions.push( Vec3::new(-size.x, 0.0, -size.y));
            normals.push(Vec3::Y);
            tangents.push(Vec4::X);
            uvs.push(Vec2::ZERO);



            positions.push( Vec3::new(size.x, 0.0, -size.y));
            normals.push(Vec3::Y);
            tangents.push(Vec4::X);
            uvs.push(Vec2::new(1.0); 0.0));



            positions.push( Vec3::new(-size.x, 0.0, size.y));
            normals.push(Vec3::Y);
            tangents.push(Vec4::X);
            uvs.push(Vec2::new(0.0); 1.0));



            positions.push( Vec3::new(size.x, 0.0, size.y));
            normals.push(Vec3::Y);
            tangents.push(Vec4::X);
            uvs.push(Vec2::ONE);



        mesh.indices.push(0);
        mesh.indices.push(2);
        mesh.indices.push(1);

        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(3);

        mesh
    }
    */
}
