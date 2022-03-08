mod time;

pub use time::*;

use ike_app::{App, CoreStage, Plugin};

#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(self, app: &mut App) {
        app.init_resource::<Time>();

        app.add_system_to_stage(Time::system, CoreStage::Start);
    }
}
