mod assets;
mod event;
mod handle;

pub use assets::*;
pub use event::*;
pub use handle::*;

use ike_app::{App, CoreStage, Plugin};
use ike_ecs::StageLabel;

#[derive(StageLabel, Clone, Copy, Debug, Hash)]
pub enum AssetStage {
    AssetEvent,
}

pub trait AddAsset: Sized {
    fn add_asset<T: Asset>(&mut self) -> &mut Self;
}

impl AddAsset for App {
    fn add_asset<T: Asset>(&mut self) -> &mut Self {
        self.add_system_to_stage(AssetEvent::<T>::system, AssetStage::AssetEvent);

        self.world.insert_resource(Assets::<T>::new());

        self.add_event::<AssetEvent<T>>();

        self
    }
}

#[derive(Default)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(self, app: &mut App) {
        app.add_stage_after(AssetStage::AssetEvent, CoreStage::PostUpdate);
    }
}
