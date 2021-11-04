mod component;
mod ext;
mod system;

pub use component::*;
pub use ext::*;
pub use system::*;

use ike_core::*;

pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    #[inline]
    fn build(self, app: &mut ike_core::AppBuilder) {
        app.add_exclusive_system_to_stage(TransformSystem, stage::POST_UPDATE);
    }
}
