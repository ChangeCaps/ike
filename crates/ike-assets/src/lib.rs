mod asset_loader;
mod asset_server;
mod assets;
mod channel;
mod event;
mod fs_io;
mod handle;
mod io;

pub use asset_loader::*;
pub use asset_server::*;
pub use assets::*;
pub use channel::*;
pub use event::*;
pub use fs_io::*;
pub use handle::*;
use ike_task::TaskPool;
pub use io::*;

use ike_app::{App, CoreStage, Plugin};
use ike_ecs::StageLabel;

#[derive(StageLabel, Clone, Copy, Debug, Hash)]
pub enum AssetStage {
    UpdateStorage,
    AssetEvent,
}

pub trait AddAsset: Sized {
    fn add_asset<T: Asset>(&mut self) -> &mut Self;

    fn add_asset_loader(&mut self, asset_loader: impl AssetLoader) -> &mut Self;
}

impl AddAsset for App {
    fn add_asset<T: Asset>(&mut self) -> &mut Self {
        self.add_system_to_stage(
            Assets::<T>::update_storage_system,
            AssetStage::UpdateStorage,
        );
        self.add_system_to_stage(AssetEvent::<T>::system, AssetStage::AssetEvent);

        self.world.insert_resource(Assets::<T>::new());

        self.add_event::<AssetEvent<T>>();

        self
    }

    fn add_asset_loader(&mut self, loader: impl AssetLoader) -> &mut Self {
        self.world.resource::<AssetServer>().add_loader(loader);

        self
    }
}

#[derive(Default)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(self, app: &mut App) {
        let task_pool = app.world.resource::<TaskPool>().clone();
        app.insert_resource(AssetServer::new(FsIo, task_pool));

        app.add_stage_before(AssetStage::UpdateStorage, CoreStage::PreUpdate);
        app.add_stage_after(AssetStage::AssetEvent, CoreStage::PostUpdate);
    }
}
