use std::any::type_name;

pub trait TypeName {
    fn type_name(&self) -> &'static str;
}

impl<T> TypeName for T {
    #[inline]
    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}
