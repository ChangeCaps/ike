mod scene;

pub use scene::*;

use ike_assets::*;
use ike_core::*;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    #[inline]
    fn build(self, app: &mut AppBuilder) {
        app.add_asset::<Scene>();
    }
}
