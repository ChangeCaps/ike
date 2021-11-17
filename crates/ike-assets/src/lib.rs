mod assets;
mod handle;
mod system;

use std::any::TypeId;

pub use assets::*;
pub use handle::*;
use ike_reflect::{FromType, ReflectAppBuilderExt, ReflectInspect, TypeRegistry};
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

        {
            let mut type_registry = self.world().write_resource::<TypeRegistry>().unwrap();

            let registration = type_registry.get_mut(&TypeId::of::<Handle<T>>()).unwrap();

            registration.insert(<ReflectInspect as FromType<Handle<T>>>::from_type());
        }

        self
    }
}
