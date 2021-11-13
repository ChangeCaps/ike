use ike_core::AppBuilder;

use crate::{RegisterType, TypeRegistry};

pub trait ReflectAppBuilderExt {
    fn register<T: RegisterType>(&mut self) -> &mut Self;
}

impl ReflectAppBuilderExt for AppBuilder {
    #[inline]
    fn register<T: RegisterType>(&mut self) -> &mut Self {
        self.world_mut().init_resource::<TypeRegistry>();

        let mut type_registry = self.world().write_resource::<TypeRegistry>().unwrap();

        T::register(&mut type_registry);

        drop(type_registry);

        self
    }
}
