use ike_math::{Vec2, Vec3};

use crate::Mesh;

#[derive(Default)]
pub struct MeshTool {
    vertex_count: usize,
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uv_0: Vec<Vec2>,
    indices: Vec<u32>,
}

impl MeshTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_index(&self) -> usize {
        self.vertex_count - 1
    }

    pub fn push_vertex(&mut self, position: Vec3) {
        self.vertex_count += 1;
        self.positions.push(position);
        self.normals.push(Vec3::ZERO);
        self.uv_0.push(Vec2::ZERO);
    }

    pub fn set_normal(&mut self, normal: Vec3) {
        self.normals[self.vertex_count - 1] = normal;
    }

    pub fn set_uv_0(&mut self, uv: Vec2) {
        self.uv_0[self.vertex_count - 1] = uv;
    }

    pub fn push_index(&mut self, index: u32) {
        self.indices.push(index);
    }

    pub fn finish(self) -> Mesh {
        let mut mesh = Mesh::new();

        mesh.insert_attribute(Mesh::POSITION, self.positions);
        mesh.insert_attribute(Mesh::NORMAL, self.normals);
        mesh.insert_attribute(Mesh::UV_0, self.uv_0);

        *mesh.get_indices_mut() = self.indices;

        mesh
    }
}
