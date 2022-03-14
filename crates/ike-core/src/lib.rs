mod time;

use std::borrow::Cow;

use ike_math::{
    IVec2, IVec3, IVec4, Mat2, Mat3, Mat3A, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec3A,
    Vec4,
};
use ike_reflect::{Reflect, ReflectDeserialize};
use ike_type::TypeRegistry;
use serde::de::DeserializeOwned;
pub use time::*;

use ike_app::{App, CoreStage, Plugin};
use ike_task::TaskPool;

#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(self, app: &mut App) {
        app.init_resource::<Time>();
        app.init_resource::<TaskPool>();

        app.add_system_to_stage(Time::system, CoreStage::Start);

        let mut type_registry = app.world.resource_mut::<TypeRegistry>();
        register_core_types(&mut type_registry);
    }
}

pub fn register_core_type<T: Reflect + DeserializeOwned>(type_registry: &mut TypeRegistry) {
    let registration = type_registry.get_mut_or_insert::<T>();

    registration.insert(ReflectDeserialize::new::<T>());
}

pub fn register_core_types(type_registry: &mut TypeRegistry) {
    register_core_type::<u8>(type_registry);
    register_core_type::<u16>(type_registry);
    register_core_type::<u32>(type_registry);
    register_core_type::<u64>(type_registry);
    register_core_type::<u128>(type_registry);
    register_core_type::<usize>(type_registry);

    register_core_type::<i8>(type_registry);
    register_core_type::<i16>(type_registry);
    register_core_type::<i32>(type_registry);
    register_core_type::<i64>(type_registry);
    register_core_type::<i128>(type_registry);
    register_core_type::<isize>(type_registry);

    register_core_type::<f32>(type_registry);
    register_core_type::<f64>(type_registry);

    register_core_type::<bool>(type_registry);

    register_core_type::<Cow<'static, str>>(type_registry);
    register_core_type::<String>(type_registry);

    register_core_type::<UVec2>(type_registry);
    register_core_type::<UVec3>(type_registry);
    register_core_type::<UVec4>(type_registry);

    register_core_type::<IVec2>(type_registry);
    register_core_type::<IVec3>(type_registry);
    register_core_type::<IVec4>(type_registry);

    register_core_type::<Vec2>(type_registry);
    register_core_type::<Vec3>(type_registry);
    register_core_type::<Vec3A>(type_registry);
    register_core_type::<Vec4>(type_registry);

    register_core_type::<Mat2>(type_registry);
    register_core_type::<Mat3>(type_registry);
    register_core_type::<Mat3A>(type_registry);
    register_core_type::<Mat4>(type_registry);

    register_core_type::<Quat>(type_registry);
}
