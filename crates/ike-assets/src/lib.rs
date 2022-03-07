mod assets;
mod event;
mod handle;

pub use assets::*;
pub use event::*;
pub use handle::*;

use ike_app::{App, Plugin};
use ike_ecs::SystemFn;

pub mod stage {
    pub const ASSET_EVENT: &str = "asset_event";
}

pub trait AddAsset: Sized {
    fn add_asset<T: Asset>(&mut self) -> &mut Self;
}

impl AddAsset for App {
    fn add_asset<T: Asset>(&mut self) -> &mut Self {
        self.add_system_to_stage(AssetEvent::<T>::system.system(), stage::ASSET_EVENT);

        self.world.insert_resource(Assets::<T>::new());

        self.add_event::<AssetEvent<T>>();

        self
    }
}

#[derive(Default)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(self, app: &mut App) {
        app.add_stage_after(stage::ASSET_EVENT, ike_app::stage::POST_UPDATE);
    }
}
