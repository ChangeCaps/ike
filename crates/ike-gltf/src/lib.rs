mod gltf_mesh;
mod mesh_loader;

pub use gltf_mesh::*;
use ike_ecs::{ParallelSystemCoercion, UpdateParentSystem};
pub use mesh_loader::*;

use ike_app::{App, CoreStage, Plugin};
use ike_assets::AddAsset;

#[derive(Default)]
pub struct GltfPlugin;

impl Plugin for GltfPlugin {
    fn build(self, app: &mut App) {
        app.add_asset::<GltfMesh>();
        app.add_asset_loader(GltfMeshLoader);

        app.add_system_to_stage(
            gltf_mesh_system.before(UpdateParentSystem),
            CoreStage::PostUpdate,
        );
    }
}
