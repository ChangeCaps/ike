use std::any::{type_name, Any};

pub trait TypeName {
    fn type_name(&self) -> &'static str;
}

impl<T: 'static> TypeName for T {
    #[inline]
    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

pub trait TypeNameAny: TypeName + Any {
    fn any(&self) -> &dyn Any;
}

impl<T: TypeName + Any> TypeNameAny for T {
    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }
}
