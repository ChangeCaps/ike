pub mod random;
pub mod time;

use ike_app::{app::App, plugin::Plugin};
use ike_ecs::schedule::DefaultStage;

#[derive(Clone, Copy, Debug, Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<time::Time>();

        app.add_system_to_stage(DefaultStage::First, time::Time::update_system);
    }
}

pub mod prelude {
    pub use crate::random::{random, random_range, Rng};
    pub use crate::time::Time;
    pub use crate::CorePlugin;
}
