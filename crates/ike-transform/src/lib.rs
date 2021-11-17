mod component;
mod ext;
mod system;

pub use component::*;
pub use ext::*;
pub use system::*;

use ike_core::*;
use ike_reflect::ReflectAppBuilderExt;

pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    #[inline]
    fn build(self, app: &mut ike_core::AppBuilder) {
        app.register::<Transform>();

        app.add_system_to_stage(insert_transform_components.system(), stage::MAINTAIN);
        app.add_system_to_stage(transform_system.system(), stage::MAINTAIN);
    }
}
