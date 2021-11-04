use super::{Mesh, Vertex};
use crate::prelude::Color;
use glam::{Vec2, Vec3, Vec4};

impl Mesh<Vertex> {
    

    #[inline]
    pub fn sphere(radius: f32, radial: u32, seg: u32) -> Self {
        let mut mesh = Self::new();

        for j in 0..seg {
            let h = (j as f32 / (seg as f32 - 1.0) - 0.5) * std::f32::consts::PI;
            let t = h.cos();

            for i in 0..=radial {
                let a = i as f32 / radial as f32 * std::f32::consts::TAU;

                let normal = Vec3::new(a.cos() * t, h.sin(), a.sin() * t);

                mesh.vertices.push(Vertex {
                    position: normal * radius,
                    normal,
                    uv: Vec2::new(i as f32 / radial as f32, j as f32 / (seg as f32 - 1.0)),
                    color: Color::WHITE,
                    ..Default::default()
                });

                let j0 = j * (radial + 1) + i;
                let j1 = j * (radial + 1) + ((i + 1) % (radial + 1));
                let i0 = j0 + (radial + 1);
                let i1 = j1 + (radial + 1);

                if j < seg - 1 && i < radial {
                    mesh.indices.push(j0);
                    mesh.indices.push(i0);
                    mesh.indices.push(j1);

                    mesh.indices.push(j1);
                    mesh.indices.push(i0);
                    mesh.indices.push(i1);
                }
            }
        }

        mesh
    }
}
