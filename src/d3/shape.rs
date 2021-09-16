use super::{Mesh, Vertex};
use crate::prelude::Color;
use glam::{Vec2, Vec3};

impl Mesh<Vertex> {
    #[inline]
    pub fn cube(mut size: Vec3) -> Self {
        size /= 2.0;

        let mut mesh = Self::new();

        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, size.y, -size.z),
            normal: Vec3::Y,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, size.y, -size.z),
            normal: Vec3::Y,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, size.y, size.z),
            normal: Vec3::Y,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 1.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, size.y, size.z),
            normal: Vec3::Y,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 1.0),
        });

        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, -size.y, -size.z),
            normal: -Vec3::Y,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, -size.y, -size.z),
            normal: -Vec3::Y,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, -size.y, size.z),
            normal: -Vec3::Y,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 1.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, -size.y, size.z),
            normal: -Vec3::Y,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 1.0),
        });

        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, -size.y, -size.z),
            normal: Vec3::X,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, -size.y, size.z),
            normal: Vec3::X,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, size.y, -size.z),
            normal: Vec3::X,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 1.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, size.y, size.z),
            normal: Vec3::X,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 1.0),
        });

        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, -size.y, -size.z),
            normal: -Vec3::X,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, -size.y, size.z),
            normal: -Vec3::X,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, size.y, -size.z),
            normal: -Vec3::X,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 1.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, size.y, size.z),
            normal: -Vec3::X,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 1.0),
        });

        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, -size.y, size.z),
            normal: Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, size.y, size.z),
            normal: Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, -size.y, size.z),
            normal: Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 1.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, size.y, size.z),
            normal: Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 1.0),
        });

        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, -size.y, -size.z),
            normal: -Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, size.y, -size.z),
            normal: -Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, -size.y, -size.z),
            normal: -Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 1.0),
        });
        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, size.y, -size.z),
            normal: -Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 1.0),
        });

        mesh.indices.push(0);
        mesh.indices.push(2);
        mesh.indices.push(1);
        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(3);

        mesh.indices.push(4);
        mesh.indices.push(5);
        mesh.indices.push(6);
        mesh.indices.push(6);
        mesh.indices.push(5);
        mesh.indices.push(7);

        mesh.indices.push(8);
        mesh.indices.push(10);
        mesh.indices.push(9);
        mesh.indices.push(9);
        mesh.indices.push(10);
        mesh.indices.push(11);

        mesh.indices.push(12);
        mesh.indices.push(13);
        mesh.indices.push(14);
        mesh.indices.push(14);
        mesh.indices.push(13);
        mesh.indices.push(15);

        mesh.indices.push(16);
        mesh.indices.push(18);
        mesh.indices.push(17);
        mesh.indices.push(17);
        mesh.indices.push(18);
        mesh.indices.push(19);

        mesh.indices.push(20);
        mesh.indices.push(21);
        mesh.indices.push(22);
        mesh.indices.push(22);
        mesh.indices.push(21);
        mesh.indices.push(23);

        mesh
    }

    /// Create quad on the xy plane.
    #[inline]
    pub fn quad(size: Vec2) -> Self {
        let mut mesh = Self::new();

        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, -size.y, 0.0),
            normal: Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::ZERO,
        });

        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, -size.y, 0.0),
            normal: Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(1.0, 0.0),
        });

        mesh.vertices.push(Vertex {
            position: Vec3::new(-size.x, size.y, 0.0),
            normal: Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::new(0.0, 1.0),
        });

        mesh.vertices.push(Vertex {
            position: Vec3::new(size.x, size.y, 0.0),
            normal: Vec3::Z,
            color: Color::WHITE,
            uv: Vec2::ONE,
        });

        mesh.indices.push(0);
        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(3);

        mesh
    }
}
