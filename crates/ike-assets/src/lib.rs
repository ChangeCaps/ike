mod assets;
mod handle;

pub use assets::*;
pub use handle::*;

use ike_app::{App, Plugin};

pub trait AddAsset: Sized {
    fn add_asset<T: Asset>(&mut self) -> &mut Self;
}

impl AddAsset for App {
    fn add_asset<T: Asset>(&mut self) -> &mut Self {
        self.world.insert_resource(Assets::<T>::new());

        self
    }
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(self, _app: &mut App) {}
}
