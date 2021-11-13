mod assets;
mod handle;
mod system;

pub use assets::*;
pub use handle::*;
use ike_reflect::ReflectAppBuilderExt;
pub use system::*;

use ike_core::FnSystem;

pub trait AssetAppBuilderExt {
    fn add_asset<T: Send + Sync + 'static>(&mut self) -> &mut Self;
}

impl AssetAppBuilderExt for ike_core::AppBuilder {
    #[inline]
    fn add_asset<T: Send + Sync + 'static>(&mut self) -> &mut Self {
        self.world_mut().insert_resource(Assets::<T>::new());
        self.add_system(asset_system::<T>.system());
        self.register::<Handle<T>>();
        self
    }
}
