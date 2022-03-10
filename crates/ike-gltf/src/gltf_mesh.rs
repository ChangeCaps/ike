use ike_assets::{Assets, Handle};
use ike_ecs::{Commands, Entity, EntityCommands, Query, Res};
use ike_render::Mesh;
use ike_transform::Transform;

#[derive(Clone, Debug)]
pub struct GltfMesh {
    pub nodes: Vec<GltfMeshNode>,
}

impl GltfMesh {
    pub fn apply(&self, entity: &EntityCommands) {
        entity.with_children(|parent| {
            for node in self.nodes.iter() {
                node.apply(&parent.spawn());
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct GltfPrimitive {
    pub mesh: Handle<Mesh>,
    #[cfg(feature = "pbr")]
    pub material: Handle<ike_pbr::PbrMaterial>,
}

#[derive(Clone, Debug)]
pub struct GltfMeshNode {
    pub transform: Transform,
    pub primitives: Vec<GltfPrimitive>,
    pub children: Vec<GltfMeshNode>,
}

impl GltfMeshNode {
    pub fn apply(&self, entity: &EntityCommands) {
        entity.insert(self.transform);

        entity.with_children(|parent| {
            for primitive in self.primitives.iter() {
                let entity = parent.spawn();

                entity.insert(Transform::IDENTITY);
                entity.insert(primitive.mesh.clone());
                #[cfg(feature = "pbr")]
                entity.insert(primitive.material.clone());
            }

            for child in self.children.iter() {
                child.apply(&parent.spawn());
            }
        });
    }
}

pub fn gltf_mesh_system(
    commands: Commands,
    gltf_meshes: Res<Assets<GltfMesh>>,
    query: Query<(Entity, &Handle<GltfMesh>)>,
) {
    for (entity, gltf_mesh_handle) in query.iter() {
        if let Some(gltf_mesh) = gltf_meshes.get(gltf_mesh_handle) {
            gltf_mesh.apply(&commands.entity(&entity));

            commands.remove::<Handle<GltfMesh>>(&entity);
        }
    }
}
