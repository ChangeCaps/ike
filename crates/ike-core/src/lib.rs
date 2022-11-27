pub mod time;

use ike_app::{App, Plugin};
use ike_ecs::schedule::DefaultStage;

#[derive(Clone, Copy, Debug, Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<time::Time>();

        app.add_system_to_stage(DefaultStage::First, time::Time::update_system);
    }
}
