#[cfg(feature = "serialize")]
mod deserialize;
mod reflect;
mod reflect_enum;
mod reflect_list;
mod reflect_map;
mod reflect_set;
mod reflect_struct;
mod reflect_tuple;
mod reflect_value;
#[cfg(feature = "serialize")]
mod serialize;

#[cfg(feature = "serialize")]
pub use deserialize::*;
pub use reflect::*;
pub use reflect_enum::*;
pub use reflect_list::*;
pub use reflect_map::*;
pub use reflect_set::*;
pub use reflect_struct::*;
pub use reflect_tuple::*;
pub use reflect_value::*;
#[cfg(feature = "serialize")]
pub use serialize::*;

#[cfg(feature = "serialize")]
pub mod type_field {
    pub const TYPE: &str = "type";
    pub const VARIANT: &str = "variant";

    pub const TUPLE: &str = "tuple";
    pub const STRUCT: &str = "struct";
    pub const ENUM: &str = "enum";
    pub const LIST: &str = "list";
    pub const SET: &str = "set";
    pub const MAP: &str = "map";
    pub const VALUE: &str = "value";
}
