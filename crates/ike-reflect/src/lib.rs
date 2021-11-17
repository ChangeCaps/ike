#![deny(unsafe_op_in_unsafe_fn)]

mod deserialize;
mod egui_value;
mod enum_trait;
mod ext;
mod map;
mod reflect;
mod serialize;
mod struct_trait;
mod tuple_struct;
mod type_registry;
mod value;

pub use deserialize::*;
pub use egui_value::*;
pub use enum_trait::*;
pub use ext::*;
pub use ike_derive::Reflect;
pub use map::*;
pub use reflect::*;
pub use serialize::*;
pub use struct_trait::*;
pub use tuple_struct::*;
pub use type_registry::*;
pub use value::*;

mod type_field {
    pub const TYPE: &str = "type";
    pub const STRUCT: &str = "struct";
    pub const TUPLE_STRUCT: &str = "tuple_struct";
    pub const VALUE: &str = "value";
}
